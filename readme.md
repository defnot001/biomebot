# Discord Bot for the Biome Toolchain Discord Server

Join the [Biome Discord](https://discord.gg/VXz9TAxHmK) for discussion about features.

_Note: This bot is only a proof-of concept and is currently not being used. There is no affiliation between this repository and the Biome maintainers. I am just implementing features that users would want in this bot to show what it would look like in Serenity/Poise._

### Todo List:

- [x] **Webserver**: Filter github webhook events and only forward the ones that came from human users.
- [x] **Command**: `languages` shows the support level of Biome's supported languages by scraping the website.
- [x] **Webserver**: Post issues into a special channel when a `good-first-issue` label gets added to it.
- [x] **Command**: `embed` command to post rich embeds to a discord webhook. Can be useful for displaying rules in a nicer way. Defaults to admin only.
