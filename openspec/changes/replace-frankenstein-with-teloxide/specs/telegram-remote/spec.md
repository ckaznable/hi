_No spec-level changes. This is a pure implementation-level dependency swap (frankenstein → teloxide) with no behavioral requirement differences. The Telegram remote adapter's external behavior — long-polling, per-chat sessions, message splitting at 4096 chars, rate-limit retry — remains identical._

_No new capabilities are introduced and no existing capability requirements are modified._
