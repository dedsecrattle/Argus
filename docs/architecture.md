# Argus Architecture

Seed URLs enter the frontier.
The frontier groups URLs by host.
Workers fetch only when host politeness rules allow.
Fetched pages are parsed for links.
Links are normalized and deduplicated.
Unseen links go back into the frontier.
