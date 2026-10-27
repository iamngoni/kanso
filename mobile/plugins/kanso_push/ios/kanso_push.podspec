Pod::Spec.new do |s|
  s.name             = 'kanso_push'
  s.version          = '0.1.0'
  s.summary          = "Kanso's push wake-up: a token to register and a silent push to sync on."
  s.description      = <<-DESC
Asks the OS for a push token and hands it to Dart, and calls the app's wake hook for each silent push. A push carries nothing.
                       DESC
  s.homepage         = 'https://kansoapp.app'
  s.license          = { :file => '../LICENSE' }
  s.author           = { 'Kanso' => 'kanso@kansoapp.app' }
  s.source           = { :path => '.' }
  s.source_files = 'kanso_push/Sources/kanso_push/**/*'
  s.dependency 'Flutter'
  s.platform = :ios, '13.0'
  s.pod_target_xcconfig = { 'DEFINES_MODULE' => 'YES', 'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386' }
  s.swift_version = '5.0'
end
