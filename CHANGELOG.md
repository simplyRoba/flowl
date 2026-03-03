# Changelog

## [0.20.1](https://github.com/simplyRoba/flowl/compare/v0.20.0...v0.20.1) (2026-03-03)


### Bug Fixes

* apply EXIF orientation when generating thumbnails ([23589f5](https://github.com/simplyRoba/flowl/commit/23589f52ba6d5e9c8eeac87778a2ab40817372e6))

## [0.20.0](https://github.com/simplyRoba/flowl/compare/v0.19.0...v0.20.0) (2026-03-03)


### Features

* add image thumbnail generation for optimized photo display ([9c4d60f](https://github.com/simplyRoba/flowl/commit/9c4d60fc5b9cd71067f81532ec23c3fa411daac3))


### Bug Fixes

* add missing doc comments for clippy pedantic ([7fdd95b](https://github.com/simplyRoba/flowl/commit/7fdd95b1db4cc3f5c907f2879a4b12b9b75316c0))
* **deps:** bump @sveltejs/kit from 2.53.3 to 2.53.4 in /ui ([14a4e98](https://github.com/simplyRoba/flowl/commit/14a4e983b176cd413c785cfd1c556035fd45beee))
* **deps:** bump lucide-svelte from 0.575.0 to 0.576.0 in /ui ([04ead3b](https://github.com/simplyRoba/flowl/commit/04ead3b7e007987df6e1374870385e92d57c17c4))
* **deps:** bump svelte from 5.53.5 to 5.53.6 in /ui ([346b25a](https://github.com/simplyRoba/flowl/commit/346b25a9c3ad743cf810383ddbd22968942dead0))
* **deps:** bump zip from 8.1.0 to 8.2.0 ([a26f0e7](https://github.com/simplyRoba/flowl/commit/a26f0e76ccf6004b04f615011cd57b722cfab510))
* log spawn_blocking errors and prevent onerror loops on thumbnails ([54a6845](https://github.com/simplyRoba/flowl/commit/54a684500b4260d01eaf7c5ff335876430d98dd6))
* move PlantForm prop initialization from $effect to onMount ([0a8feaa](https://github.com/simplyRoba/flowl/commit/0a8feaadc68bb0f8c4c63cd78074d682285d8147))
* use boolean flag for note error styling instead of string comparison ([f84f08d](https://github.com/simplyRoba/flowl/commit/f84f08ddb4919bd2cdc144a358ae408179172bcb))
* validate imported data in restore path using shared validators ([69f029c](https://github.com/simplyRoba/flowl/commit/69f029cb77fadc66057d1e8b12e6755813be9dd0))

## [0.19.0](https://github.com/simplyRoba/flowl/compare/v0.18.1...v0.19.0) (2026-03-01)


### Features

* **ui:** add confirm dialog before deleting a care entry ([5e5b920](https://github.com/simplyRoba/flowl/commit/5e5b92056a825a61d58760758dbc572590bdc208))


### Bug Fixes

* make data restore atomic by writing photos before DB commit ([b478fbc](https://github.com/simplyRoba/flowl/commit/b478fbc710423b0c01694d3d389472a086623aa1))
* prevent frozen buttons on API error in plant detail ([f7396b1](https://github.com/simplyRoba/flowl/commit/f7396b1a4ac24d67ac702959a100f16e4b8c8b0d))
* **ui:** allow text selection on care journal entries ([9fc86f4](https://github.com/simplyRoba/flowl/commit/9fc86f42019170cbb7ae2e6962216fbe0508fefc))
* update care entry delete test for confirm dialog and fix a11y ([1df81d0](https://github.com/simplyRoba/flowl/commit/1df81d07022f2c0af0dc4a431a6522790754b5b2))
* update care entry form tests to match refactored component ([9510ba1](https://github.com/simplyRoba/flowl/commit/9510ba112f3381d7be7fa2e94309085d0fc2199e))
* use bound parameters for event_type in list_all_care_events ([79202b3](https://github.com/simplyRoba/flowl/commit/79202b3641008c0b110d6dec5ee0bfb117a47904))

## [0.18.1](https://github.com/simplyRoba/flowl/compare/v0.18.0...v0.18.1) (2026-02-28)


### Bug Fixes

* **ui:** add inline icons to care entry type chips ([27ea19e](https://github.com/simplyRoba/flowl/commit/27ea19ec6e53723a3fe1383a4b097ffe9deb6513))
* **ui:** align timeline icon, text, and delete button vertically ([39e4888](https://github.com/simplyRoba/flowl/commit/39e4888203775f3534e3fe3eec6cddec9d7f029b))
* **ui:** care journal mobile improvements ([d149bb1](https://github.com/simplyRoba/flowl/commit/d149bb14ae46dcf4ba17d3e0bd2d6b2592f628bb))
* **ui:** pin lightbox dialog to viewport for PWA safe-area offset ([82333fe](https://github.com/simplyRoba/flowl/commit/82333fe72438a99d2c2b369d732826387302ffac))

## [0.18.0](https://github.com/simplyRoba/flowl/compare/v0.17.0...v0.18.0) (2026-02-28)


### Features

* add care event photo support with shared ImageStore ([40dcd99](https://github.com/simplyRoba/flowl/commit/40dcd9971bfb46e3b7700aae8db56c430b54afc7))
* add care event photo support with shared ImageStore ([e0523a2](https://github.com/simplyRoba/flowl/commit/e0523a2c3c57399ea8fa56308de736c26e7f6639))


### Bug Fixes

* **ui:** offset photobox close button by safe-area inset in PWA mode ([5650296](https://github.com/simplyRoba/flowl/commit/565029630d7155fdc9ac71f128ca12a8111fa1a1))
* **ui:** use AI accent color for enabled status dot on settings page ([d1e97a6](https://github.com/simplyRoba/flowl/commit/d1e97a69d280c16c986b2f436f976bfa3d0751c5))
* **ui:** widen chat drawer on large desktop screens ([f085b7b](https://github.com/simplyRoba/flowl/commit/f085b7be72a274b9894138088b301da5e617c114))

## [0.17.0](https://github.com/simplyRoba/flowl/compare/v0.16.5...v0.17.0) (2026-02-28)


### Features

* **ui:** add photo attachment to AI chat ([7680dd2](https://github.com/simplyRoba/flowl/commit/7680dd270831842230ff1f4a062ea05be59c0ac4))


### Bug Fixes

* **api:** replace missing_errors_doc suppressions with proper doc comments ([5ea9221](https://github.com/simplyRoba/flowl/commit/5ea9221dbd206dd90a9aadb317a2ebf312ddbc1e))
* **ui:** close drawer after saving note and reload care events ([095b5a5](https://github.com/simplyRoba/flowl/commit/095b5a5e4110489f3208c37440ee03d00a06bd76))
* **ui:** prevent summary textarea overflow on mobile ([65634c3](https://github.com/simplyRoba/flowl/commit/65634c33d1760b680d9b589871891404f862e98b))
* **ui:** show water button label on mobile and add spacing to status badge ([f402ca8](https://github.com/simplyRoba/flowl/commit/f402ca87840fe06038b5f923d4cdf56d0b8f029c))

## [0.16.5](https://github.com/simplyRoba/flowl/compare/v0.16.4...v0.16.5) (2026-02-27)


### Bug Fixes

* **ui:** add bottom spacing between content and mobile nav bar ([8b00957](https://github.com/simplyRoba/flowl/commit/8b0095787080e7d85f62fcc7ce8497c33d9c6d43))
* **ui:** use border-box on chat drawer sheet for safe-area padding ([6d62062](https://github.com/simplyRoba/flowl/commit/6d62062dacc1dd8a19f84c05d73c2fc301059446))

## [0.16.4](https://github.com/simplyRoba/flowl/compare/v0.16.3...v0.16.4) (2026-02-27)


### Bug Fixes

* **ui:** add safe-area padding to mobile chat drawer ([ef55989](https://github.com/simplyRoba/flowl/commit/ef55989d2b2c027e6ae401354d5fe58eb7106a9a))

## [0.16.3](https://github.com/simplyRoba/flowl/compare/v0.16.2...v0.16.3) (2026-02-27)


### Bug Fixes

* **ui:** use content-height for nav bar, let padding-bottom add safe area ([87e39a4](https://github.com/simplyRoba/flowl/commit/87e39a4462a81e3c4162e6fa5e990c59b3502605))

## [0.16.2](https://github.com/simplyRoba/flowl/compare/v0.16.1...v0.16.2) (2026-02-27)


### Bug Fixes

* **ui:** centralize safe-area-bottom into global CSS variables ([c942149](https://github.com/simplyRoba/flowl/commit/c9421491432b52a7f393c2de421d08603cecf801))

## [0.16.1](https://github.com/simplyRoba/flowl/compare/v0.16.0...v0.16.1) (2026-02-27)


### Bug Fixes

* **ui:** add safe-area-inset-bottom for PWA mode on iPhone ([089d693](https://github.com/simplyRoba/flowl/commit/089d6933986202247d2ab49d94e52d07174813db))

## [0.16.0](https://github.com/simplyRoba/flowl/compare/v0.15.5...v0.16.0) (2026-02-27)


### Features

* add chat summary save-note flow with ai-consultation event type ([d850f65](https://github.com/simplyRoba/flowl/commit/d850f65954c78730d12211d436159430d6e24e2c))
* add photo count to settings data stats ([f12ba15](https://github.com/simplyRoba/flowl/commit/f12ba157de416e32dc1ed0e8c3c778dc52bf9d82))
* **ui:** add PWA web app manifest for home screen installability ([c32ff1d](https://github.com/simplyRoba/flowl/commit/c32ff1df137c6557fe418421426ec09bdcd6548d))


### Bug Fixes

* **i18n:** use proper plural forms for delete location verb ([641032d](https://github.com/simplyRoba/flowl/commit/641032da92e0508694057c4da6870cab753747d8))
* **ui:** improve settings page mobile layout ([15d3149](https://github.com/simplyRoba/flowl/commit/15d31490d0d242d41e6097ca4f7edc1f1ea67bfa))

## [0.15.5](https://github.com/simplyRoba/flowl/compare/v0.15.4...v0.15.5) (2026-02-27)


### Bug Fixes

* **ui:** set theme-color meta tag for iOS Safari browser chrome ([6779164](https://github.com/simplyRoba/flowl/commit/6779164d2a51f3f2ea5b376933f4f2f87950d33e))

## [0.15.4](https://github.com/simplyRoba/flowl/compare/v0.15.3...v0.15.4) (2026-02-27)


### Bug Fixes

* **ci:** bump actions/download-artifact from 7 to 8 ([32cb6ae](https://github.com/simplyRoba/flowl/commit/32cb6ae965fa32327f1fa700bf9d88c2b56296df))
* **ci:** bump actions/upload-artifact from 6 to 7 ([3298e70](https://github.com/simplyRoba/flowl/commit/3298e704370fab2676e71a5920108d81c1edfafc))
* **deps:** bump @sveltejs/kit from 2.53.2 to 2.53.3 in /ui ([18b5833](https://github.com/simplyRoba/flowl/commit/18b5833a02be88ea3e94c280d188927999ed9374))
* **deps:** bump svelte-check from 4.4.3 to 4.4.4 in /ui ([e1955b9](https://github.com/simplyRoba/flowl/commit/e1955b9e9af238b16bb8e02add77d33d6d45faf9))
* **ui:** add spacing between identify section and name field ([b1949ba](https://github.com/simplyRoba/flowl/commit/b1949badc8d6b335ba0ffad2aec1805bbcb5aa7f))
* **ui:** ensure detail-info takes full width on mobile ([6d1784e](https://github.com/simplyRoba/flowl/commit/6d1784e22d1ff77d3b622829d8d5656269249fc1))
* **ui:** migrate to body scroll for iOS Safari chrome collapse ([ff6a06c](https://github.com/simplyRoba/flowl/commit/ff6a06cad112b6cc69cfbb85e08720f7bf1cebd5))

## [0.15.3](https://github.com/simplyRoba/flowl/compare/v0.15.2...v0.15.3) (2026-02-26)


### Bug Fixes

* **ui:** hide empty assistant bubble while streaming ([1c0c1ca](https://github.com/simplyRoba/flowl/commit/1c0c1cad8b05c7e68e65e99ec7f5f2ade0d2640e))

## [0.15.2](https://github.com/simplyRoba/flowl/compare/v0.15.1...v0.15.2) (2026-02-26)


### Bug Fixes

* **ui:** make hero action buttons equal width on mobile ([892a681](https://github.com/simplyRoba/flowl/commit/892a681d38c29286baf72a4bd3afc7cf94059585))
* **ui:** restore sticky header, add drawer spacing, hide action bar ([3432ef3](https://github.com/simplyRoba/flowl/commit/3432ef3da20098264902aabc5693ea8f4cd207d2))

## [0.15.1](https://github.com/simplyRoba/flowl/compare/v0.15.0...v0.15.1) (2026-02-26)


### Bug Fixes

* **ui:** disable pinch-to-zoom on mobile ([6e7e00b](https://github.com/simplyRoba/flowl/commit/6e7e00b7c4eca8ae72272d44686191bfb44d80db))
* **ui:** extend mobile chat drawer over bottom navigation ([8229740](https://github.com/simplyRoba/flowl/commit/822974033860d20dc33ce962471f75d193d7afca))
* **ui:** prevent iOS Safari auto-zoom on input focus ([1580c13](https://github.com/simplyRoba/flowl/commit/1580c139b5e7927e394b5026c9841e2f66fd391d))
* **ui:** render chat drawer side-by-side on desktop/tablet ([747975c](https://github.com/simplyRoba/flowl/commit/747975c1bab9d4667b5f7d0196b433f0dfd42ebd))
* **ui:** use --fs-input variable for chat input font size ([f793070](https://github.com/simplyRoba/flowl/commit/f793070e86c7e678de24f9b63bfcebfef2abf807))

## [0.15.0](https://github.com/simplyRoba/flowl/compare/v0.14.1...v0.15.0) (2026-02-26)


### Features

* **ai:** add streaming chat and summarize endpoints ([5279fbd](https://github.com/simplyRoba/flowl/commit/5279fbddcfe78639d3f2c2ee4be2ceb862cdbeed))
* **ai:** return up to 3 ranked identification suggestions per request ([eeee4d1](https://github.com/simplyRoba/flowl/commit/eeee4d10a0c937d7897851b368fe9b70a5f515a3))
* **ui:** add AI chat drawer to plant detail page ([320cd95](https://github.com/simplyRoba/flowl/commit/320cd9532884f0b7debaec73b3d5dba53dfbca74))


### Bug Fixes

* **deps:** bump rollup from 4.57.1 to 4.59.0 in /ui ([318338e](https://github.com/simplyRoba/flowl/commit/318338e277dbf7c6849524fa2832bdacda160381))
* resolve clippy pedantic lints for CI ([b8cc141](https://github.com/simplyRoba/flowl/commit/b8cc14166a7a1f2b5c0aca7adb2df177d1d7fcbd))

## [0.14.1](https://github.com/simplyRoba/flowl/compare/v0.14.0...v0.14.1) (2026-02-25)


### Bug Fixes

* **deps:** bump @sveltejs/kit from 2.53.1 to 2.53.2 in /ui ([#47](https://github.com/simplyRoba/flowl/issues/47)) ([906e54f](https://github.com/simplyRoba/flowl/commit/906e54f4478987ab098d53e78f3e4cda071f6fcc))
* **deps:** bump svelte from 5.53.3 to 5.53.5 in /ui ([#46](https://github.com/simplyRoba/flowl/issues/46)) ([b58b91b](https://github.com/simplyRoba/flowl/commit/b58b91bf6a7151dd8ce2b80f59e5b6d2e115bf7a))
* **ui:** prevent care journal date input from overflowing container on mobile ([9dfce10](https://github.com/simplyRoba/flowl/commit/9dfce10909d5fdc32eb98de13e80b3547d5e5cef))
* **ui:** show main photo inline in identify extra-photos row instead of duplicating at top ([bc866ca](https://github.com/simplyRoba/flowl/commit/bc866ca68f1723ef1c8472940bb56c9942240ed5))
* **ui:** truncate long watering interval labels on mobile with ellipsis ([14efa02](https://github.com/simplyRoba/flowl/commit/14efa0257b8b67253b2d6d5e081d551456bd0cc2))

## [0.14.0](https://github.com/simplyRoba/flowl/compare/v0.13.0...v0.14.0) (2026-02-25)


### Features

* **ai:** add debug logging to plant identification flow ([9274f7f](https://github.com/simplyRoba/flowl/commit/9274f7fae7f1d891b96cf179c0440bed77dd7b56))
* **ai:** localize plant identification results based on user locale ([81f8567](https://github.com/simplyRoba/flowl/commit/81f85678288d7d076c85d3ddab950106dc332bef))
* **ai:** switch to structured output with enum constraints and update default model to gpt-4.1-mini ([ec70969](https://github.com/simplyRoba/flowl/commit/ec7096976a71769e77060bdad71b1ead104be940))
* **settings:** implement backend persistence for user preferences (theme, locale) with REST API ([397f147](https://github.com/simplyRoba/flowl/commit/397f1478c1d8adbd440fcf0bffb840c02ef8a1a3))
* **ui:** move identify section from Media to Identity section ([5588825](https://github.com/simplyRoba/flowl/commit/5588825b158500b62c9a770bd0d0398335bc4629))


### Bug Fixes

* **deps:** bump @sveltejs/kit from 2.53.0 to 2.53.1 in /ui ([d7fafe7](https://github.com/simplyRoba/flowl/commit/d7fafe7d9d148f388665ec4a6c13d60c8cd57815))
* **ui:** prevent care journal date input from overflowing container on mobile ([d22dcbf](https://github.com/simplyRoba/flowl/commit/d22dcbf8062352ba3c711663d0a02e74102833c3))
* use valid MQTT wildcard in repair discovery subscribe ([31e3031](https://github.com/simplyRoba/flowl/commit/31e3031e95da74aabf87dc34592635aa02c45f2f))

## [0.13.0](https://github.com/simplyRoba/flowl/compare/v0.12.0...v0.13.0) (2026-02-24)


### Features

* **ai:** add AI provider foundation with status endpoint ([4ecb73e](https://github.com/simplyRoba/flowl/commit/4ecb73e1bd790e8b66e6294090a881ea213f5270))
* **api:** add POST /api/ai/identify endpoint for plant identification ([593b1fb](https://github.com/simplyRoba/flowl/commit/593b1fbcfc225c6e60fb63431fc9b42dc9cb8100))
* **ui:** add AI assistant status section to settings page ([aad1185](https://github.com/simplyRoba/flowl/commit/aad1185d172c8226b2fac9027f682719d13e90aa))
* **ui:** add plant identification UI to PlantForm ([be96965](https://github.com/simplyRoba/flowl/commit/be96965279548bdfa800163a64be011ab60fa0e5))


### Bug Fixes

* **deps:** bump @sveltejs/kit from 2.52.2 to 2.53.0 in /ui ([d571b9b](https://github.com/simplyRoba/flowl/commit/d571b9b867533293ad816992c355f2e15577d4ed))
* **deps:** bump chrono from 0.4.43 to 0.4.44 ([b470602](https://github.com/simplyRoba/flowl/commit/b470602e964a38a88ac32ca8abf09e6b713fea0e))
* **deps:** bump svelte from 5.53.0 to 5.53.2 in /ui ([09b41a9](https://github.com/simplyRoba/flowl/commit/09b41a963fa91725f486672a7bc3cd0eeae9f438))
* **deps:** bump svelte-check from 4.4.1 to 4.4.3 in /ui ([6dc1214](https://github.com/simplyRoba/flowl/commit/6dc12148a242809a0c4ecd4f24f535a8a74daf90))
* **deps:** bump zip from 2.4.2 to 8.1.0 ([dd04186](https://github.com/simplyRoba/flowl/commit/dd041866db6e110b1a845bce417be2fac4cb8a96))

## [0.12.0](https://github.com/simplyRoba/flowl/compare/v0.11.0...v0.12.0) (2026-02-22)


### Features

* add HTTP access log middleware ([475c962](https://github.com/simplyRoba/flowl/commit/475c962eee2114b9e21d387d15d26dc33b17377b))
* add location_count to stats and update related interfaces and tests ([612eabc](https://github.com/simplyRoba/flowl/commit/612eabca5b57b7efa7533c74c863abdea9f26810))
* add logging to API handlers and MQTT publish operations ([2b222ad](https://github.com/simplyRoba/flowl/commit/2b222adfa2117c404f7879d1493c38469dde4d85))
* compute last_watered from care_events instead of stored column ([886c672](https://github.com/simplyRoba/flowl/commit/886c67256bde2e81e11ab76762140bd14c93e7f8))
* enhance settings page with descriptions for Repair and Backup options ([b361e8d](https://github.com/simplyRoba/flowl/commit/b361e8d221f7be3ab8222eff1300d52a77380f90))
* **i18n:** add internationalization support with de, en, es ([#35](https://github.com/simplyRoba/flowl/issues/35)) ([bc3cdd5](https://github.com/simplyRoba/flowl/commit/bc3cdd512261d6fff8f982a8edb9549018918174))
* replace native confirm() with themed ModalDialog component ([4bcacdd](https://github.com/simplyRoba/flowl/commit/4bcacdd91e5e149afe6cae5446c355f69107f022))


### Bug Fixes

* delete location immediately if it has no plants ([7535381](https://github.com/simplyRoba/flowl/commit/7535381821ec66f0c84a78857b2d0e68e7e9f974))
* mqtt repair event loop ([#34](https://github.com/simplyRoba/flowl/issues/34)) ([22f98f6](https://github.com/simplyRoba/flowl/commit/22f98f6ee452bce55a5f88dc122138d27c941bcb))
* prevent lightbox dialog from blocking page content when closed ([eb066cf](https://github.com/simplyRoba/flowl/commit/eb066cf9e337055e0a82e473c23bac1ab096df39))
* prevent portrait images from stretching attention cards ([a4d502d](https://github.com/simplyRoba/flowl/commit/a4d502d4251959a69d7d44b92be778cdadef5082))
* **ui:** fix position of success and error message on settings actions ([e0a00f6](https://github.com/simplyRoba/flowl/commit/e0a00f6514561017c2af5a705ca01848d7b09c4c))
* update import message format to include locations ([de684c0](https://github.com/simplyRoba/flowl/commit/de684c0b2cf20404de731a45b3c8231d4fe6fbe5))

## [0.11.0](https://github.com/simplyRoba/flowl/compare/v0.10.0...v0.11.0) (2026-02-21)


### Features

* detail image lightbox zoom ([#32](https://github.com/simplyRoba/flowl/issues/32)) ([e17fb34](https://github.com/simplyRoba/flowl/commit/e17fb344d5d640bbc78e15ad54568178f8ad69f0))
* MQTT repair endpoint with orphan cleanup and auto-reconnect republish ([d00c37f](https://github.com/simplyRoba/flowl/commit/d00c37f4fefd4183dc8abe78b8f3a1e20051d09a))


### Bug Fixes

* **docs:** update commands in review expectations and config for consistency ([10a51db](https://github.com/simplyRoba/flowl/commit/10a51dbd1720a4218463a26a893f7a5e926554e4))
* **plan:** mark seasonal watering adjustments as won't implement for now ([695047f](https://github.com/simplyRoba/flowl/commit/695047faf8a55fc2de2de14e78de249d2bae4f69))
* remove unused variable and unnecessary drop in backup_restore tests ([2e975b7](https://github.com/simplyRoba/flowl/commit/2e975b7340407a0dbd93765ee087448f89e0623d))
* **ui:** adjust dimensions of photo upload and detail components for better display ([dc60c0b](https://github.com/simplyRoba/flowl/commit/dc60c0b053175c9c43bd9748483cea454a6b74b9))
* **ui:** match stepper input height to buttons on mobile ([c61be1d](https://github.com/simplyRoba/flowl/commit/c61be1dcc5df2af7d1afcefb3ed97a4ef0e4466c))
* **ui:** update back label in PageHeader from "Back" to "Cancel" ([9b05f64](https://github.com/simplyRoba/flowl/commit/9b05f642948a4aa6b0d9c06a879dec30ba893f7e))
* **ui:** update plant links to include source query parameter for navigation ([ef1c313](https://github.com/simplyRoba/flowl/commit/ef1c313e4ae5f95af94e9507b7c3197dc7fc45a9))

## [0.10.0](https://github.com/simplyRoba/flowl/compare/v0.9.0...v0.10.0) (2026-02-20)


### Features

* add About section to settings page with app info endpoint ([0b9722b](https://github.com/simplyRoba/flowl/commit/0b9722baa576ae0ea8c961d7b8083e548e2daa7f))
* add Data section to settings page with plant and care entry counts ([e060b57](https://github.com/simplyRoba/flowl/commit/e060b57afc8af376af81670f68d84ca5a0726e71))
* add MQTT status section to settings page ([15abd88](https://github.com/simplyRoba/flowl/commit/15abd885c37780d0e289ab76eb312db1bd999436))


### Bug Fixes

* **ci:** bump actions/setup-node from 4 to 6 ([887dc1a](https://github.com/simplyRoba/flowl/commit/887dc1a6cc72101a18c50c44eb93ad37694352ee))
* **deps:** bump lucide-svelte from 0.574.0 to 0.575.0 in /ui ([7085f38](https://github.com/simplyRoba/flowl/commit/7085f38a68a7c2e3ca5d054a2e5b68c048af6256))

## [0.9.0](https://github.com/simplyRoba/flowl/compare/v0.8.0...v0.9.0) (2026-02-19)


### Features

* add care info attributes to plants (difficulty, pet safety, growth speed, soil type, soil moisture) ([a847c93](https://github.com/simplyRoba/flowl/commit/a847c93e2d447f3c44a48f5086eec5cc28ffee41))
* **ui:** add dashboard "Needs Attention" section with inline water action ([cd53d70](https://github.com/simplyRoba/flowl/commit/cd53d70f104f6cd0bfe5598fe1f983a264a66581))
* **ui:** add icons to watering card interval, last watered, and next due fields ([118f535](https://github.com/simplyRoba/flowl/commit/118f535074e635106302fae50b6c805fe14d25a3))


### Bug Fixes

* **deps:** bump @sveltejs/kit from 2.52.0 to 2.52.2 in /ui ([ae1095c](https://github.com/simplyRoba/flowl/commit/ae1095c99c3970cce7bed4f9c9a4d71738b4bda2))
* **deps:** bump svelte from 5.51.3 to 5.53.0 in /ui ([73314b7](https://github.com/simplyRoba/flowl/commit/73314b75128d467da10d93f2d62f826738c8c6ec))
* **deps:** bump svelte-check from 4.4.0 to 4.4.1 in /ui ([1b3e035](https://github.com/simplyRoba/flowl/commit/1b3e035a30d031003e4fde0e2638a8280781eb4e))
* **ui:** add missing care info fields to test mock plants ([70bb3f6](https://github.com/simplyRoba/flowl/commit/70bb3f6d204c8a923dc693f4fd8c0e180c1f450b))
* **ui:** remove border from care journal delete button ([a1d9a3d](https://github.com/simplyRoba/flowl/commit/a1d9a3ddd9eb9fb45df2ad7793aaf26af3e1af57))

## [0.8.0](https://github.com/simplyRoba/flowl/compare/v0.7.0...v0.8.0) (2026-02-18)


### Features

* add inline location rename in settings ([69e7fba](https://github.com/simplyRoba/flowl/commit/69e7fba36bf41873dc64768cdd1268de9aeae7ec))
* add reusable PageHeader with sticky desktop header and mobile action bar ([73f5259](https://github.com/simplyRoba/flowl/commit/73f5259336249cc55bfdf730848631465da674fa))
* add short year to care journal dates ([1ef5a1a](https://github.com/simplyRoba/flowl/commit/1ef5a1a910173ed49e9d809598e340e1b6d97c32))
* add time-based greeting with random variations to dashboard ([783f543](https://github.com/simplyRoba/flowl/commit/783f543415aa6e475d24c174ff5b8ac5547474ab))
* extract StatusBadge component and apply overlay-style dashboard cards ([d837500](https://github.com/simplyRoba/flowl/commit/d8375007ea0e26ab52773439466dafc2b330fc50))
* implement consistent content width tokens across application ([f35cb1e](https://github.com/simplyRoba/flowl/commit/f35cb1e6ff54118ff7ef78834d32fbda91299104))


### Bug Fixes

* add global min-width to prevent layout collapse at narrow viewports ([f7375c1](https://github.com/simplyRoba/flowl/commit/f7375c1bf409f86bfb4244aa6ee88a2aff203b97))
* add vertical padding to PageHeader for top breathing room ([3d298b1](https://github.com/simplyRoba/flowl/commit/3d298b14e768a8628ad0a26f672e4dbe0ff0fc08))
* **deps:** bump jsdom from 26.1.0 to 28.1.0 in /ui ([#17](https://github.com/simplyRoba/flowl/issues/17)) ([9f3ab14](https://github.com/simplyRoba/flowl/commit/9f3ab1489239ccfaa883bc50e460e77a4f405351))
* **deps:** bump lucide-svelte from 0.564.0 to 0.568.0 in /ui ([#18](https://github.com/simplyRoba/flowl/issues/18)) ([42047b1](https://github.com/simplyRoba/flowl/commit/42047b1c85adad9575a6ad1faae469621f67c1b3))
* **deps:** bump lucide-svelte from 0.569.0 to 0.574.0 in /ui ([0f1fc14](https://github.com/simplyRoba/flowl/commit/0f1fc14d7a6f9b7f7ab4c9019687722875ee1036))
* **deps:** bump svelte from 5.51.2 to 5.51.3 in /ui ([fbc389a](https://github.com/simplyRoba/flowl/commit/fbc389abc682c36d6335c69fd09992e93a1b0090))
* **deps:** bump vitest from 3.2.4 to 4.0.18 in /ui ([#19](https://github.com/simplyRoba/flowl/issues/19)) ([4ddbc4b](https://github.com/simplyRoba/flowl/commit/4ddbc4b1a7592809523dbb7f0dbd977dcdf48edd))
* remove redundant theme hint from settings page ([f9eb090](https://github.com/simplyRoba/flowl/commit/f9eb0900b026f97fa6b3d2d5e3d1e1f1962c60cd))
* responsive plant card text sizes, grid columns, and name-badge spacing ([bff659f](https://github.com/simplyRoba/flowl/commit/bff659f7aa755812476f65ab488ab1fbd0a2bf58))
* unify page heading sizes to 22px across all pages ([752ef8d](https://github.com/simplyRoba/flowl/commit/752ef8db2a2a27f182ba1220667c269c0182f1d6))
* use theme-aware backgrounds for StatusBadge in dark mode ([4919806](https://github.com/simplyRoba/flowl/commit/4919806472a646ae28331a4c69619cf07ef5b5d1))

## [0.7.0](https://github.com/simplyRoba/flowl/compare/v0.6.0...v0.7.0) (2026-02-16)


### Features

* add care journal delete controls ([dc8922c](https://github.com/simplyRoba/flowl/commit/dc8922c95d32e0be13ee92a019c90491f66e965f))


### Bug Fixes

* sort care events by occurred date ([8930104](https://github.com/simplyRoba/flowl/commit/89301045b663265dcd055d39f9f13f386676fd33))

## [0.6.0](https://github.com/simplyRoba/flowl/compare/v0.5.0...v0.6.0) (2026-02-16)


### Features

* add dark mode theming and settings ([#15](https://github.com/simplyRoba/flowl/issues/15)) ([4945bfe](https://github.com/simplyRoba/flowl/commit/4945bfe689b0b792fc97e56e6e5e00288d84380b))
* align plant detail layout with mockup ([6cd3e7a](https://github.com/simplyRoba/flowl/commit/6cd3e7a4a21931771a8e7f60242ea154b1b7fea9))
* allow backdated care log entries ([59ac6e5](https://github.com/simplyRoba/flowl/commit/59ac6e55cb14e9fa0a012012724c4d9a9bf6f914))
* enhance plant detail layout with improved styling and icons ([dc29d5b](https://github.com/simplyRoba/flowl/commit/dc29d5b957530a03df99b74bde8161f9d81ada0e))
* enhance UI components and improve user experience ([fe144c0](https://github.com/simplyRoba/flowl/commit/fe144c0eb48e31bf7d9d70622485c81e363464ff))
* improve care journal UI and enhance date formatting ([d5291a2](https://github.com/simplyRoba/flowl/commit/d5291a25827061a20729f151577db09a0eb50fc4))
* introduce mqtt disabled flag ([4971fd2](https://github.com/simplyRoba/flowl/commit/4971fd2f26646c469b26ba0ef8d45e46a83fbe8a))
* update date formatting for care events and improve date parsing in UI ([8ee9106](https://github.com/simplyRoba/flowl/commit/8ee91065780c6907e616656872707e95a0b94c77))


### Bug Fixes

* align media action buttons ([a2cec4e](https://github.com/simplyRoba/flowl/commit/a2cec4e09e38c5e60b2b3e75deee2cbe07fd97b8))
* **db:** ensure parent directories are created for SQLite database path ([e89069a](https://github.com/simplyRoba/flowl/commit/e89069af260f8998ee5d87b693c20a8b18ae3605))
* enable host option for development server in package.json ([013359e](https://github.com/simplyRoba/flowl/commit/013359e09d21d2ef3499c3497c3999839994d58b))

## [0.5.0](https://github.com/simplyRoba/flowl/compare/v0.4.0...v0.5.0) (2026-02-16)


### Features

* add care journal with event tracking, timeline UI, and global log ([79f9ece](https://github.com/simplyRoba/flowl/commit/79f9ece837c8201fb419f68f84fde3da3f6c8f86))
* add dev server with hot reloading for UI and backend ([0b3bf80](https://github.com/simplyRoba/flowl/commit/0b3bf8059d8f8ee3e6b3e047c999f8adbc804414))
* add Playwright MCP server for headless browser access ([3d513b4](https://github.com/simplyRoba/flowl/commit/3d513b45cfa4a7666c7aa98ab9f490de751f59dd))


### Bug Fixes

* add .DS_Store to .gitignore to ignore Mac system files ([b1e0929](https://github.com/simplyRoba/flowl/commit/b1e0929ddc39e92bd9c2707ab601c5ad9222ed5f))
* **deps:** bump @sveltejs/kit from 2.51.0 to 2.52.0 in /ui ([a919869](https://github.com/simplyRoba/flowl/commit/a9198696e7c837a2d1d99e3e789bc9858f208021))
* **deps:** bump svelte from 5.51.1 to 5.51.2 in /ui ([14b7407](https://github.com/simplyRoba/flowl/commit/14b74073478819bc4da286fa961d9b45c8ac5441))
* format SQL query string for better readability ([45357ec](https://github.com/simplyRoba/flowl/commit/45357ec2ed11d03d386904109525b707ed3f3533))
* opencode configuration for Playwright MCP server ([568a463](https://github.com/simplyRoba/flowl/commit/568a4632cbc624216feb2c047b58e44fbfcc44c8))
* update npm initialization command in devcontainer configuration ([071d73b](https://github.com/simplyRoba/flowl/commit/071d73bca91fba45395705ddf87f5269916ef822))
* update README for backend command and remove obsolete UI README ([fefb42a](https://github.com/simplyRoba/flowl/commit/fefb42a2883180f65b8fb0a407068e6cb16d83de))
* update rust feature configuration and refine npm commands in devcontainer ([8282b89](https://github.com/simplyRoba/flowl/commit/8282b89060eefc784a1647f4871e83dfd00d8f40))

## [0.4.0](https://github.com/simplyRoba/flowl/compare/v0.3.0...v0.4.0) (2026-02-15)


### Features

* add watering lifecycle, MQTT state publishing, and UI indicators ([aad9124](https://github.com/simplyRoba/flowl/commit/aad9124ed61d025228b8113a0163b62c33691b88))

## [0.3.0](https://github.com/simplyRoba/flowl/compare/v0.2.4...v0.3.0) (2026-02-15)


### Features

* add plant photo upload and settings page ([141ca66](https://github.com/simplyRoba/flowl/commit/141ca667d29b3f76fe6a6d84b78d73aa49448993))
* **mockups:** add plant photo examples and fix detail photo shape ([acd4967](https://github.com/simplyRoba/flowl/commit/acd49679bbb37c08467852361ea397c84503c160))
* **ui:** add custom flowl owl logo ([c7f7363](https://github.com/simplyRoba/flowl/commit/c7f736374cff4831b01088260b79589406f91f26))
* **ui:** add widescreen layout with expanded sidebar and overlay cards ([7475d95](https://github.com/simplyRoba/flowl/commit/7475d957ba5b1ad272d76b8cb0969759b0403b3b))


### Bug Fixes

* **ui:** highlight active nav item based on current route ([98cfaba](https://github.com/simplyRoba/flowl/commit/98cfabaee2ce228d022bad7d5eaacf8504c4fa58))
* **ui:** make notes textarea full width in plant form ([9b0d2d9](https://github.com/simplyRoba/flowl/commit/9b0d2d927b920f9d9153a5e569f24257500ba6ec))
* **ui:** match mobile bottom nav to mockup design ([a108997](https://github.com/simplyRoba/flowl/commit/a1089972018156fc1199ce94782859349d5a00f7))

## [0.2.4](https://github.com/simplyRoba/flowl/compare/v0.2.3...v0.2.4) (2026-02-15)


### Bug Fixes

* create /data directory in container for SQLite database ([39b8434](https://github.com/simplyRoba/flowl/commit/39b843434387a0977d23d9e0105a843ac4ee95e8))

## [0.2.3](https://github.com/simplyRoba/flowl/compare/v0.2.2...v0.2.3) (2026-02-15)


### Bug Fixes

* use debian trixie for glibc 2.38 compatibility ([03f1a5a](https://github.com/simplyRoba/flowl/commit/03f1a5ae0b334217ef1dc52f3f808e4a540710e2))

## [0.2.2](https://github.com/simplyRoba/flowl/compare/v0.2.1...v0.2.2) (2026-02-15)


### Bug Fixes

* **ci:** use target-specific CC for cross-compilation ([d25dfc3](https://github.com/simplyRoba/flowl/commit/d25dfc33dc5064379f4fb1858f242e727ac66906))

## [0.2.1](https://github.com/simplyRoba/flowl/compare/v0.2.0...v0.2.1) (2026-02-15)


### Bug Fixes

* **ci:** add npm dependabot for ui dependencies ([c25fce0](https://github.com/simplyRoba/flowl/commit/c25fce06235a97763a1e9a0fde70380b47ca6521))
* **ci:** remove cross-compiler from CI and isolate cache keys ([091523d](https://github.com/simplyRoba/flowl/commit/091523dcfa038613d07e6f18bb3272d069f68282))
* **deps:** bump rumqttc from 0.24.0 to 0.25.1 ([#5](https://github.com/simplyRoba/flowl/issues/5)) ([dbe5972](https://github.com/simplyRoba/flowl/commit/dbe59724f5119fdd3b6b6a69b1e1a7f3c4df1f03))
* **deps:** bump svelte from 5.51.0 to 5.51.1 in /ui ([#4](https://github.com/simplyRoba/flowl/issues/4)) ([1a1772f](https://github.com/simplyRoba/flowl/commit/1a1772fb1f4fa19623d67e0807e5b47e365575dc))
* resolve double scrollbar on mobile in mockups and app ([7b8342e](https://github.com/simplyRoba/flowl/commit/7b8342e143142c8a8cca75e67da5d9f5f09d9584))

## [0.2.0](https://github.com/simplyRoba/flowl/compare/v0.1.0...v0.2.0) (2026-02-14)


### Features

* add permissions configuration and Live Server extension to devcontainer ([f003392](https://github.com/simplyRoba/flowl/commit/f0033920f83b0a00c87b78d0e82751b95e0a5ad4))
* add phase-1 foundation (Axum server, SQLite, MQTT, SvelteKit shell) ([1930710](https://github.com/simplyRoba/flowl/commit/193071065019009d2aed94039a2c3df5f1bedfa7))
* add plant and location CRUD with full UI ([9cc7ffd](https://github.com/simplyRoba/flowl/commit/9cc7ffd95d933427486c5daa050d18b9b03a1444))
* remove Live Server extension from devcontainer configuration ([e508f6b](https://github.com/simplyRoba/flowl/commit/e508f6b2172767cccdd0e6557ff9bf0f5759dda0))
* replace emoji with Lucide and Noto Color Emoji icons ([174e4c2](https://github.com/simplyRoba/flowl/commit/174e4c21a58e43b92282a1cf34195046817062a6))
* update Lucide icon implementation and enhance mobile responsiveness ([054a47f](https://github.com/simplyRoba/flowl/commit/054a47f7babb9a7c67843563c821508cd8b3a8fd))

## 0.1.0 (2026-02-14)


### Features

* add CI/CD workflows, Dockerfile, and project tooling ([4c70de5](https://github.com/simplyRoba/flowl/commit/4c70de5bf448479ecd1e4d4ef2c9f9c903076002))
