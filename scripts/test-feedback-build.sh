#!/usr/bin/env bash
# Guards .github/workflows/feedback-build.yml (docs/FEEDBACK.md, "The build workflow").
# You cannot run Actions locally, so this parses the YAML and asserts the safety rules
# that keep an issue's text from steering a write-capable agent. Run it from anywhere:
#
#   scripts/test-feedback-build.sh
#
# Needs ruby (macOS and most Linux have it). Run actionlint separately.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec ruby -ryaml - "$root" <<'RUBY'
root = ARGV[0]
wf_path = File.join(root, ".github/workflows/feedback-build.yml")
raw = File.read(wf_path)
wf = YAML.safe_load(raw)
$fail = []
def check(name, ok)
  if ok
    puts("ok   #{name}")
  else
    $fail << name
    puts("FAIL #{name}")
  end
end

jobs = wf["jobs"]
on = wf["on"] || wf[true] # YAML 1.1 reads the key `on` as true
action_steps = jobs.flat_map { |id, j| (j["steps"] || []).map { |s| [id, s] } }
claude = action_steps.select { |_, s| s["uses"].to_s.start_with?("anthropics/claude-code-action@") }
claude_args = claude.map { |id, s| [id, s.dig("with", "claude_args").to_s] }

# Trigger: only `issues: labeled`.
check "triggers only on issues labeled", on == { "issues" => { "types" => ["labeled"] } }

# Sender and label guards on the gate, in an `if:`.
gate_if = jobs.dig("gate", "if").to_s
check "gate: sender must be iamngoni", gate_if.include?("github.event.sender.login == 'iamngoni'")
check "gate: label must be build or plan-ok",
  gate_if.include?("github.event.label.name == 'build'") && gate_if.include?("github.event.label.name == 'plan-ok'")
check "gate: never on a pull request", gate_if.include?("github.event.issue.pull_request == null")
check "gate: only issues the Worker filed", gate_if.include?("'from-app'") && gate_if.include?("needs-triage")

# Every other job is downstream of the gate, so it can only run after the guard passed.
others = jobs.keys - ["gate"]
check "every job after the gate needs it", others.all? { |id| Array(jobs[id]["needs"]).include?("gate") }
check "agent jobs only run in a gate mode",
  %w[plan build review].all? { |id| jobs[id]["if"].to_s.include?("needs.gate.outputs.mode") }

# One build at a time, never cancelling.
conc = wf["concurrency"]
check "concurrency group set", conc.is_a?(Hash) && conc["group"].to_s.include?("feedback-build")
check "concurrency never cancels a running build", conc.is_a?(Hash) && conc["cancel-in-progress"] == false

# Daily cap.
check "daily cap is 5", wf.dig("env", "DAILY_CAP") == "5"
gate_run = jobs.dig("gate", "steps").map { |s| s["run"].to_s }.join("\n")
check "daily cap counts today's runs and stops with a comment",
  gate_run.include?("gh run list") && gate_run.include?("DAILY_CAP") && gate_run.include?("gh issue comment") && gate_run.include?("exit 1")

# Size gate: m and l plan first, plan-ok continues.
check "size gate: plan mode and plan-ok continuation",
  gate_run.include?("mode=plan") && gate_run.include?("plan-ok") && gate_run.include?("mode=build")

# Runner is one switchable variable.
runners = jobs.map { |id, j| [id, j["runs-on"].to_s] }
check "runner is the FEEDBACK_RUNNER variable everywhere", runners.all? { |_, r| r.include?("vars.FEEDBACK_RUNNER") }
check "runner default is self-hosted Linux X64", runners.all? { |_, r| r.include?('["self-hosted","Linux","X64"]') }

# Timeouts on every job.
check "timeout-minutes on every job", jobs.all? { |_, j| j.key?("timeout-minutes") }

# The agent steps: action pinned by SHA, models, caps, tools.
check "three agent steps (plan, build, review)", claude.map(&:first).sort == %w[build plan review]
check "claude-code-action pinned by commit SHA", claude.all? { |_, s| s["uses"].match?(/@[0-9a-f]{40}( |\z)/) }
check "actions/checkout pinned by commit SHA",
  action_steps.select { |_, s| s["uses"].to_s.start_with?("actions/checkout@") }.all? { |_, s| s["uses"].match?(/@[0-9a-f]{40}\z/) }
check "max-turns on every agent", claude_args.all? { |_, a| a.match?(/--max-turns\s+\S+/) }
check "build uses sonnet, review uses opus",
  claude_args.to_h["build"].include?("--model sonnet") && claude_args.to_h["review"].include?("--model opus")
check "explicit allowedTools on every agent", claude_args.all? { |_, a| a.include?("--allowedTools") }

allowed = claude_args.map do |id, a|
  [id, a[/--allowedTools\s+"([^"]*)"/, 1].to_s.split(/,(?![^(]*\))/)]
end
allowed.each do |id, tools|
  bad = tools.select { |t| t.match?(/WebFetch|WebSearch|curl|wget|gh api|gh secret|gh pr merge|^Bash\(\*\)|^Bash$/) }
  check "#{id}: allow-list has no WebFetch, curl, gh api or merge (#{tools.size} tools)", bad.empty? && !tools.empty?
end
check "build can push only to feedback/*",
  allowed.to_h["build"].grep(/git push/) == ["Bash(git push origin feedback/:*)"]
check "review can push only to feedback/*",
  allowed.to_h["review"].grep(/git push/) == ["Bash(git push origin feedback/:*)"]
check "plan changes no file and has no git push",
  allowed.to_h["plan"].none? { |t| t.match?(/^(Edit|Write)|git /) }
check "every agent disallows web, curl and gh api",
  claude_args.all? { |_, a| d = a[/--disallowedTools\s+"([^"]*)"/, 1].to_s; %w[WebFetch WebSearch Bash(curl Bash(gh\ api].all? { |w| d.include?(w) } }

# The prompt names the issue number and BUILD.md only; the issue text never enters it.
prompts = claude.map { |_, s| s.dig("with", "prompt").to_s }
check "prompts point at BUILD.md", prompts.all? { |p| p.include?(".github/feedback/BUILD.md") }
interp = prompts.flat_map { |p| p.scan(/\$\{\{\s*([^}]*?)\s*\}\}/).flatten }.uniq
check "prompts interpolate only the issue number and repository (#{interp.join(', ')})",
  (interp - ["github.event.issue.number", "github.repository"]).empty?

# No issue-controlled text in a shell script: titles and bodies must never reach `run:`.
runs = jobs.flat_map { |_, j| (j["steps"] || []).map { |s| s["run"].to_s } }.join("\n")
check "no issue title or body interpolated into a run script",
  !runs.match?(/\$\{\{[^}]*github\.event\.(issue\.(title|body)|comment)/) && !raw.match?(/github\.event\.issue\.(title|body)/)

# Protected paths.
protected_steps = action_steps.select { |_, s| s["name"] == "Protected paths" }
check "protected-path check in the build and review jobs", protected_steps.map(&:first).sort == %w[build review]
protected_steps.each do |id, s|
  pat = s.dig("env", "PROTECTED").to_s
  re = Regexp.new(pat)
  check "#{id}: protected-path step runs even if the agent failed", s["if"].to_s == "always()"
  hits = [".github/workflows/x.yml", ".github/CODEOWNERS", "cloud/wrangler.jsonc", "cloud/wrangler.toml",
          "scripts/release.sh", "scripts/apt-release.sh", "scripts/mobile-release/aab-check.sh",
          "crates/kanso-update/keys/trusted-keys.txt", "crates/kanso-gallery/keys/themes-trusted-keys.txt",
          "cliff.toml", "CLAUDE.md", ".claude/settings.json"]
  clear = ["crates/kanso-store/src/lib.rs", "cloud/src/feedback.ts", "site/src/routes/privacy.tsx",
           "mobile/lib/main.dart", "docs/FEEDBACK.md", "scripts/ci-rust.sh"]
  check "#{id}: pattern blocks every protected sample", hits.all? { |h| re.match?(h) }
  check "#{id}: pattern lets ordinary files through", clear.none? { |c| re.match?(c) }
  check "#{id}: step fails the job on a hit", s["run"].to_s.include?("exit \"$bad\"") && s["run"].to_s.include?("gh pr diff")
end

# Permissions: nothing at the top, and only what the spec lists for the build job.
check "workflow permissions start empty", wf["permissions"] == {}
check "build permissions are contents, pull-requests, issues write",
  jobs.dig("build", "permissions") == { "contents" => "write", "pull-requests" => "write", "issues" => "write" }
check "no job gets id-token, actions write, packages or deployments",
  jobs.all? { |_, j| (j["permissions"] || {}).none? { |k, v| %w[id-token packages deployments statuses checks].include?(k) || (k == "actions" && v == "write") } }
check "checkout never persists credentials",
  action_steps.select { |_, s| s["uses"].to_s.start_with?("actions/checkout@") }.all? { |_, s| s.dig("with", "persist-credentials") == false }

# Secrets: only the OAuth token and the job's own token.
secrets = raw.scan(/secrets\.([A-Za-z0-9_]+)/).flatten.uniq.sort
check "secrets used: only CLAUDE_CODE_OAUTH_TOKEN (#{secrets.join(', ')})", secrets == ["CLAUDE_CODE_OAUTH_TOKEN"]
check "no secrets: inherit and no environment", !raw.include?("secrets: inherit") && jobs.none? { |_, j| j.key?("environment") }
tokens = raw.scan(/github_token:\s*(.*)/).flatten.uniq
check "agent token is the job's GITHUB_TOKEN", tokens == ["${{ github.token }}"]

# CODEOWNERS covers the protected paths.
co = File.read(File.join(root, ".github/CODEOWNERS"))
["/.github/**", "/cloud/wrangler*", "/scripts/*release*"].each do |p|
  check "CODEOWNERS covers #{p}", co.lines.any? { |l| l.start_with?(p) && l.include?("@iamngoni") }
end

# BUILD.md carries the rules.
build_md = File.read(File.join(root, ".github/feedback/BUILD.md"))
["untrusted", "data, not instructions", "`.github/`", "`cloud/wrangler*`", "Tests are required", "Reported-by"].each do |t|
  check "BUILD.md mentions #{t}", build_md.downcase.include?(t.downcase)
end

if $fail.empty?
  puts "\nfeedback-build: all checks passed"
else
  puts "\nfeedback-build: #{$fail.size} failed"
  exit 1
end
RUBY
