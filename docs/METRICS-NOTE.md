# Kanso counts, it doesn't watch

Kanso sends a small anonymous count once a day, so we know how many people use it and on which
devices. Here is everything it sends, and everything it doesn't.

## What's sent

Once a day, while Kanso is open:

- The kind of device: Mac, Windows, Linux, iPhone, iPad or Android, and its system version.
- Kanso's version.
- Whether sync is on, and whether it's Kanso's server or your own.
- Whether AI is turned on.
- Roughly how many notes you have, as a range: none, 1–9, 10–99, 100–999 or 1,000 or more.

And the first time each day you use one of these: creating a note, voice input, publishing,
importing (and which app you imported from), locking, AI, quick capture, sharing a note with
someone, or exporting. Only that it was used that day, never what with.

The country your connection comes from is added on our server. Your IP address is not kept or
passed on.

## What isn't

- Your notes, their titles, notebooks, tags, files or links
- Anything you type, search for or ask AI
- Your name, username or email address
- Your IP address
- Anything about your other apps or your computer beyond its system version

## How you're counted

If you're signed in to an account, your devices share one ID made from your account's encryption
key, so your laptop and phone count as one person. Our server never has that key, so neither we
nor anyone else can tell which account the ID belongs to. If you're not signed in, each device
has a random ID of its own.

## Where it goes

To Kanso's own server first, which checks every field against a fixed list and refuses anything
else, then on to [PostHog](https://posthog.com) (EU) to draw the charts. There's no tracking code
from anyone else in Kanso.

## Can I turn it off?

No. It's how we know Kanso is being used at all, and it's kept to the list above so it never
needs to be. Debug and development builds don't send anything.

## The code

Every detail is in [docs/METRICS.md](https://github.com/iamngoni/kanso/blob/master/docs/METRICS.md):
the exact fields and values, and links to the code that sends them and the server that checks them.
