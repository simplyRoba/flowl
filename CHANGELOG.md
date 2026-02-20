# Changelog

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
