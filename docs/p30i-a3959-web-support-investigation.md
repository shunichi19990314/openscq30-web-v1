# Soundcore P30i (A3959) — v1 Web版サポート追加 調査報告と実装方針

作成日: 2026-09-23
対象: OpenSCQ30 v1 Webクライアント（Web Bluetooth + WASM）
参照: OpenSCQ30 master（v2）`lib/src/devices/soundcore/a3959*`、`tools/soundcore-device-faker/devices/a3959.toml`、v1（`origin/v1`）

---

## 0. エグゼクティブサマリ

- **v2のA3959実装は「2バイトコマンド＋共通ヘッダ」のパケット言語**で書かれており、v1の「7バイトCommand定長ヘッダ込み定数」とも**同一のワイヤプロトコル**である（ヘッダの抽象化レベルが違うだけ）。したがって**プロトコル层面での移植障壁は低い**。
- v2がWebを廃止したのは「P30iがBLE GATTを持たないから」ではなく、**v2が接続バックエンドをRFCOMM（Classic BT）専用に統一したため**（`connection_backend/` に linux/windows とも rfcomm しか無い）。Web BluetoothはGATTしか話せないためv2アーキテクチャとは原理的に共存不可。
- 一方v1はGATT経路（`SERVICE_UUID` マスク方式 + 特性 `0x7777`/`0x8888`）を保持。**P30iが同GATTサービスを持つなら（Soundcore iOSアプリがP30iをサポートする＝iOSはGATT必須のため、持つ可能性が極めて高い）、v1 Webでのサポートは実現可能**。
- 推奨方針: **方針A「v1 libへの移植（段階的UI拡張）」**。コア機能（接続・状態表示・ANC系・EQ・バッテリー）→ 拡張機能（ボタン設定・サラウンド・ゲーミングモード等）の2段階。

---

## 1. 公式v2（master）のA3959実装調査

### 1.1 ファイル構成とアーキテクチャ

```
lib/src/devices/soundcore/a3959.rs                  # soundcore_device! マクロによるデバイス定義
lib/src/devices/soundcore/a3959/modules.rs
lib/src/devices/soundcore/a3959/modules/sound_modes.rs
lib/src/devices/soundcore/a3959/modules/sound_modes/setting_handler.rs
lib/src/devices/soundcore/a3959/packets.rs
lib/src/devices/soundcore/a3959/packets/inbound.rs
lib/src/devices/soundcore/a3959/packets/inbound/state_update.rs
lib/src/devices/soundcore/a3959/state.rs
lib/src/devices/soundcore/a3959/structures.rs
lib/src/devices/soundcore/a3959/structures/sound_modes.rs
```

- `soundcore_device!` マクロに ①状態構造体 ②初期状態取得クロージャ（`RequestState` 送信→_state update_受信→必要ならデュアル接続デバイス取得）③モジュール登録クロージャ ④faker用デフォルト応答、を渡す構成。
- 登録（`device_model.rs`）: `DeviceModel::SoundcoreA3959` → `new_soundcore_device!(soundcore::a3959)`。表示名はi18n（`lib/i18n/*/openscq30-lib.ftl`）で "Soundcore P30i / R50i NC"。
- 使用モジュール（=サポート機能）:
  - `add_state_update` / `a3959_sound_modes`（**独自** SoundModes: Manual/Adaptive/MultiScene + 風切り音 + 感度）
  - `equalizer_with_drc_tws(common_settings_type_2())`（**1ch×10バンド + DRC**）
  - `button_configuration`（8ボタン/4設定エントリ、**ボタン単位パケット** `[0x04,0x81]`、`supports_set_all_packet: false`、`ignore_enabled_flag: true`）
  - `ambient_sound_mode_cycle` / `reset_button_configuration` / `dual_connections`（マルチポイント）/ `surround_sound` / `auto_power_off`(10/20/30/60分) / `touch_tone` / `tws_status` / `low_battery_prompt` / `gaming_mode`
  - `dual_battery(10)`（**0–10スケール**）/ `serial_number_and_dual_firmware_version`
- **ファームウェア gating**: `gaming_mode` は `dual_firmware_version.min() >= 1.60` のときのみ有効（テスト `has_no_gaming_mode_on_old_firmware` で担保）。

### 1.2 ワイヤフレーミング（v2 vs v1）

| | v2 (master) | v1 |
|---|---|---|
| コマンド抽象 | `Command([u8;2])`（例: `[0x01,0x01]`） | 7バイト定数 `Command::new([hdr5bytes..., cmd2])`（例: STATE_UPDATE = `[0x09,0xff,0,0,1, 0x01,0x01]`） |
| ヘッダ | `common/packet.rs` が方向バイト・長さ・チェックサム（`ChecksumKind`）を動的生成 | パケット種別ごとにヘッダ込み定数（送信 `0x08,0xee,0,0,0` / 受信 `0x09,0xff,0,0,1` 等） |
| 状態要求 | `RequestState` = Command `[0x01,0x01]` | `RequestStatePacket` wire = `[0x08,0xee,0,0,0, 0x01,0x01, 0x0a,0x00,0x02]` |

→ **実体は同一プロトコル**。v1へ移植する際は「v1流のヘッダ込みCommand定数」を該当パケット分に定義するだけでよい（長さバイトの計算規則はv1既存パケットの例から機械的に導出可能）。

### 1.3 A3959 ステートアップデートパケット バイトマップ（faker toml + parse順から復元）

受信コマンド `[0x01,0x01]`、本体約89–90バイト（ファームによる）。offsetは0始まり:

| off | size | 字段 | 備考 |
|---|---|---|---|
| 0 | 1 | host device | どちらのイヤホンがホストか |
| 1 | 1 | TWS接続 | 0=片側のみ |
| 2–3 | 2 | バッテリL/R | **0–10スケール**（fakerコメント: 0=10%…9=100%） |
| 4–5 | 2 | 不明(255,255) | |
| 6–15 | 10 | 左FWバージョン ASCII | "01.64" 形式 |
| 16–25 | 10 | 右FWバージョン ASCII | |
| 26–41 | 16 | シリアル ASCII | |
| 42–43 | 2 | EQプロファイル | 254=カスタム系フラグ |
| 44–53 | 10 | EQバンド×10 | **エンコード: byte = value + 120**（valueは0.1dB単位らしきi16、テスト [-19,0,41,…] ↔ [101,120,161,…] で確認） |
| 54–63 | 10 | DRC関連ブロック | v2でもTODOあり（"last 3 bytes of DRC are off by 1"）。**現状は不明ブロックとしてpreserve** |
| 64 | 1 | 不明(10) | faker: 変更するとゲーミングモード/アンビエントが壊れる → **そのまま送り返す必須バイト** |
| 65–72 | 8 | ボタン動作 L/R×(1,2,3,ロング) | `ActionKind::TwsLowBits`、0xFF=無効 |
| 73 | 1 | アンビエントサイクル | |
| 74 | 1 | AmbientSoundMode | Normal/Transparency/NoiseCanceling |
| 75 | 1 | Manual+Adaptive複合バイト | 上位/下位ニブル（0x55=manual5/adaptive5） |
| 76 | 1 | AmbientSoundMode（重複） | |
| 77 | 1 | NCモード種別 | 0=Manual 1=Adaptive 2=MultiScene |
| 78 | 1 | 風切り音低減 | |
| 79 | 1 | 不明(255) | parse上は adaptive sensitivity level 位置 |
| 80 | 1 | MultiScene ANC | 0=Transport 1=Outdoor 2=Indoor |
| 81 | 1 | 不明(49) | |
| 82 | 1 | タッチトーン | |
| 83 | 1 | デュアル接続（マルチポイント） | |
| 84 | 1 | 不明 | parse順では SurroundSound の位置 |
| 85–86 | 2 | 自動電源OFF 有効/時間 | |
| 87 | 1 | 低バッテリー通知 | |
| 88 | 1 | ゲーミングモード | fw<1.60では無効扱い |
| 89–100 | 12 | 不明(0) | |

### 1.4 設定系コマンド（テストから確認できたもの）

| 設定 | コマンド | ペイロード例 | 備考 |
|---|---|---|---|
| 状態要求 | `[0x01,0x01]` | v1 wire: `…0x0a,0x00,0x02` | |
| サウンドモード系（Manual NC等） | `[0x06,0x81]` | SoundModes 7バイト（`[0,37,0,0,1,255,1]`等） | sound_modes_v2共通モジュール経由 |
| EQ(+DRC) 設定 | `[0x02,0x83]` | 22バイト = プロファイル2+バンド10+DRC10 | |
| サラウンド | `[0x02,0x86]` | `[1]` | |
| ボタン動作 | `[0x04,0x81]` | `[order_idx, button_id, action]`（無効はaction=0xFF） | ボタン毎に個別送信 |
| ゲーミング/マルチポイント/自動電源OFF/タッチトーン/低バッテリー通知 | v2 commonモジュールの共通コマンド | — | 移植時にcommon実装から起こす |

### 1.5 その他特徴

- ボタン設定は「全ボタン一括パケット非対応」→ v1の `InternalMultiButtonConfiguration`（一括型）とは**送受信とも変換レイヤが必要**。
- EQは **モノラル10バンド**（v1既存はステレオ8バンド±quirk）。DRCブロックはv2ですら解釈が未完（TODO）→ v1では「受信値をそのまま送り返す」preserve戦略が安全。
- faker toml（`tools/soundcore-device-faker/devices/a3959.toml`）に**全バイトの意味コメント付きの応答例**があり、単体テストのベクタとしてそのまま使える（`rfcomm_uuid` はfakerの_transport_定義であり、デバイスがRFCOMM必須という意味ではない点に注意）。

---

## 2. v1 Web版の既存実装との比較

### 2.1 v1のアーキテクチャ（再確認）

- `lib/src/devices/<model>/{device_profile.rs, packets/...}`: モデル毎に **DeviceProfile（DeviceFeatures + compatible_models + implementation）** と nom パーサ。
- parse結果は共通の `standard::packets::inbound::state_update_packet::StateUpdatePacket` に**正規化**されて wasm/JS 側へ（serde camelCase）。
- Web UI（React）は `DeviceFeatures` と StateUpdatePacket の字段を見てコンポーネントを出し分け（`soundMode/` と `soundModeTypeTwo/` の2系統が存在）。
- Web接続: `web/wasm/src/web_bluetooth_connection.rs` — GATT接続→**最初のprimary service**→特性 `0x7777`(write) / `0x8888`(notify)。デバイスピッカーは `manufacturerData`（MAC先頭3バイト）フィルタ + `optionalServices: service_uuids()`（`SERVICE_UUID = 011cf5da-…` の先頭2バイト±32範囲）。

### 2.2 共通点（移植の追い風）

- ワイヤプロトコル同一（§1.2）。状態要求〜notify受信〜書き込み応答の流れも同一。
- v1に**既に存在する概念**: sound modes type two（Adaptive/Manual/風切り音/感度レベル）、DRCフラグ（`has_dynamic_range_compression`）、タッチトーン、自動電源OFF、アンビエントサイクル、シリアル/FW表示枠、ボタン設定UI、EQ UI（バンド数はfeatures駆動）。
- StateUpdatePacket正規化層のおかげで、**wasmバインディングとUIの大半は変更せずに済む**設計になっている。

### 2.3 相違点（作業が発生する箇所）

| 項目 | v1既存 | A3959 | 影響 |
|---|---|---|---|
| サウンドモード構造 | type2 = 6バイト（multi-scene無し、3バイト目がTransparencyMode） | 7バイト（ambient重複バイトあり、**MultiScene ANCあり**、感度バイト位置違い） | 新struct（type3相当）+ parse + UI追加（MultiScene選択） |
| EQ | ステレオ8バンド（+DRC変種、+2バンドquirk） | **モノラル10バンド + DRCブロック** | 送信パケット新変種（`[0x02,0x83]` 22B）、UIはバンド数features駆動なのでほぼ流用 |
| バッテリー | パーセント表記系 | 0–10スケール | ×10変換（v2 `dual_battery(10)` 相当） |
| ボタン設定 | 一括型 `InternalMultiButtonConfiguration` | ボタン単位 `[0x04,0x81]` | 受信8バイト→内部表現変換、送信はボタン毎ループ |
| 未搭載機能 | — | ゲーミングモード、サラウンド、マルチポイント、低バッテリー通知、MultiScene | DeviceFeatures拡張（serde camelCase追加フィールド）+ UIトグル新設 |
| FW gating | 一部あり（DRC min fw等） | ゲーミングモード≥1.60 | 既存パターンで対応可 |

### 2.4 v1 Webが持つ既存制約（P30iでも継続）

- 切断イベント無し（`connection_status` は常時Connected）、MACアドレス取得不可。
- GATTサービスは「最初のprimary service」を採用（Soundcoreサービスが複数サービス中先頭である前提）。
- ピッカーのMACプレフィスフィルタ（`device_utils::MAC_ADDRESS_PREFIXES`）にP30iのOUIが無ければ**追加が必要**（または非フィルタモードで運用）。

---

## 3. Web Bluetooth（BLE）での実現可能性

### 3.1 前提事実

- v2 masterの接続バックエンドは **RFCOMMのみ**（`connection_backend/{linux,windows}/rfcomm.rs`、`soundcore.rs` の `VENDOR_RFCOMM_UUID = 0cf12d31-…` マスク）。**これがv2にWebが無い真の理由**（Web BluetoothはGATTのみ提供し、RFCOMM/SPPは使えない）。
- v1はGATT経路を維持。Soundcore機はGATTサービスUUIDが「先頭2バイトが機種毎にインクリメント」する連続族（`device_utils` のコメント＆マスク方式）。
- P30i(A3959)はSoundcore公式モバイルアプリ（**iOS版はBLE GATT必須**）で設定可能であるため、**同GATTサービス族＋7777/8888特性を持つ可能性が極めて高い**（※実機未検証＝最大のリスク項目）。

### 3.2 実現可能な範囲（GATT経路で全てコマンドチャネル完結）

- 接続・状態取得・バッテリー・FW/シリアル表示
- ANC系（Normal/Transparency/NC、Manual強度、Adaptive感度、MultiScene、風切り音）
- EQ（10バンド+DRC preserve）、サラウンド、ゲーミングモード、マルチポイント表示/設定、タッチトーン、自動電源OFF、低バッテリー通知
- ボタン動作設定（ボタン単位送信）

### 3.3 難しい／不可な部分

- **Classic BT/RFCOMM必須の機能**: 無し（プロトコルはGATT上で完結する見込み）。ただし実機がGATTサービスを露出していない場合は根本不可（→その場合v1 WebでのP30iサポートは諦めるか、制限付き案内にする）。
- ファームウェア更新: v1 Webも未対応（スコープ外）。
- 上記§2.4の既存Web制約はそのまま。
- 実測が必要な項目: ①サービスUUIDの先頭2バイト値が `service_uuids()` の±32範囲内か（外れていればRANGE拡張）、②MACプレフィスリスト、③MTU（状態パケット~90Bはnotify分割で問題にならない見込み、v1既存機と同様）。

---

## 4. 実装方針の提案と評価

### 方針A: v1 libへA3959を移植（推奨）

v1の `lib/src/devices/` に `a3959` モジュールを新設（v2のパケット定義をv1流儀に翻訳）、`DeviceProfile`/`DeviceModel`/registryに登録、wasmはほぼ無変更、UIは差分のみ追加。

- 難易度: ★★☆（中）
- 工数目安: コア（lib+parse+テスト）1–2日 / UI差分（MultiScene・トグル群・EQ10バンド確認）1–2日 / 実機検証・デプロイ 0.5–1日
- 長所: 変更局所・v1のテスト資産（nom単体テストパターン）を流用可・UI構造を壊さない・faker tomlをテストベクタに出来る
- 短所: v2将来機能とは再び分岐（v1はメンテナンスモード相当）

### 方針B: v2 libベースの新WASMターゲット

v2 lib（settings/SettingId モデル）へ wasm-bindgen 層を新規作成、React側もsettings駆動UIへ改修。

- 難易度: ★★★★（高）
- 工数目安: 2–4週（wasmバインディング新設＋UI全面改修＋v2 libのwasm対応整備: tokio/wasm-target整備含む）
- 長所: 今後v2側の全機種・全設定がWebに乗る「根本解決」
- 短所: v1 WebのUI構造を大きく壊す（制約違反）、v2 libはwasmターゲットを持たず_ASYNC/バックエンド整理_が重い、1GB CIメモリ問題等の副次作業も増

### 方針C: JS側最小実装（lib bypass）

TS側でA3959専用ライトパス（接続・状態要求・ANC切替・EQ・バッテリーのみ）を実装。

- 難易度: ★★☆（中低）
- 工数目安: 1–2日
- 長所: 最速でコア機能だけ乗る
- 短所: プロトコルロジックがTSとRustで二重管理になる・v1 UI（StateUpdatePacket駆動）に自然に載らず別UI分岐が増える・テスト資産化しにくい → **非推奨**

### 方針D: A＋段階リリース（Aの実運用形）

Aを「Phase1=コア機能（接続/状態/ANC系/EQ/バッテリー）」「Phase2=拡張（ボタン/サラウンド/ゲーミング/マルチポイント/その他）」に分け、Phase1を先にデプロイ。

- 難易度・工数: Aと同じ（デプロイ回数が2回）
- 長所: 実機検証フィードバックを早期に得られる・リスク分断
- **実質的な推奨形**

---

## 5. 推奨方針（D=A段階移植）の具体的実装ステップ

### Phase 0: 実機検証（コードを書く前に・0.5日・実機必須）

1. Chrome/Edge の `about://bluetooth-internals` または nRF Connect でP30iをスキャン。
2. GATTサービス一覧から `xxxxf5da-0000-1000-8000-00805f9b34fb` 族のUUIDと、特性 `00007777-…` / `00008888-…` の存在を確認。
3. `0x7777` へ `[0x08,0xee,0x00,0x00,0x00,0x01,0x01,0x0a,0x00,0x02]`（v1状態要求）を書き込み、`0x8888` のnotifyで**約90バイトの状態パケット**が返るかを確認（返ってきたバイト列を記録→単体テストベクタ化）。
4. サービスUUID先頭2バイトが `service_uuids()` の範囲内か、advertiseのmanufacturerData（MACプレフィス）を確認。
   - 不合格（GATT無し）なら: 本タスクは「Web不可」と結論付け、READMEに制限記載のみ行う。

### Phase 1: libコア（v1）

1. `lib/src/soundcore_device/device_model.rs`: `A3959` 追加（シリアル12文字目以降 "3959" で自動マッチ）。
2. `lib/src/devices/a3959/` 新設:
   - `packets/state_update_packet.rs`: §1.3バイトマップのnomパーサ（不明バイトは字段として保持し送り返す）。
   - `packets/outbound/`: `set_sound_mode_type_three`（`[0x06,0x81]`+7B）、`set_equalizer_mono10_drc`（`[0x02,0x83]`+22B）、`set_button_action`（`[0x04,0x81]`+3B）、`set_surround`（`[0x02,0x86]`+1B）等。v1流ヘッダ付きCommand定数を既存パケットの規則で作る。
   - `device_profile.rs`: `A3959_DEVICE_PROFILE`（features: bands=10/ch=1/DRC=true/type3サウンドモード/ボタン=true/タッチトーン/自動電源OFF/新フラグ群）。
3. `devices.rs` / `device_profile.rs`(registry) / `DeviceFeatures`（`has_gaming_mode`, `has_surround_sound`, `has_dual_connections`, `has_low_battery_prompt`, `has_multi_scene_anc` 等をcamelCaseで追加）/ `StateUpdatePacket` 拡張字段。
4. バッテリー0–10→パーセント変換、FW≥1.60ゲーミングモードgate。
5. 単体テスト: faker tomlの応答バイト列＋v2テストのベクタ（issue 149パケット等）を `#[test]` に移植。

### Phase 2: wasm + UI差分

1. wasm: 原則変更無し（StateUpdatePacket/DeviceFeaturesのserde拡張が自動反映）。必要なら `soundcore_device_utils` 調整（MACプレフィス追加はlib側）。
2. UI:
   - `soundModeTypeTwo/` を拡張するか `soundModeTypeThree/` を新設: MultiScene選択（Transport/Outdoor/Indoor）、Adaptive感度スライダ表示。
   - DeviceSettings にサラウンド/ゲーミング/マルチポイント/低バッテリー通知のトグル行を追加（features駆動で既存機種には表示されない）。
   - EQ: 10バンド表示の確認（`num_equalizer_bands` 駆動のはず）、DRCは既存UI流用。
   - ボタン設定: 既存 `ButtonSettings` UIを流用（送信層でボタン単位ループ）。
3. i18n: 新規ラベルはv1 webのlocales仕組み（i18next）に追加。

### Phase 3: 検証・デプロイ

1. `cargo test`（lib）、`npm run test`（vitest）、必要ならPlaywright e2e（既存機種のリグレッション）。
2. 実機でコア操作を検証（Phase 1機能→Phase 2機能の順）。
3. デプロイ: openscq30-web-v1 リポジトリのワークフロー（commit `01b5337` 版、wasm-packはプリビルドインストーラ導入済み）を復元してpush → Actionsビルド → dist反映。**注意: 現在のmainにはワークフローが存在しない（dist置換時に自己削除された）ため、`.github/workflows/build-upstream.yml` の復元コミットが必須**。

### Phase 4（任意）: faker整備

- v1リポジトリ側にfakerは無いが、開発용で master の `tools/soundcore-device-faker` を借用し、a3959.tomlをGATTエミュレーション用に調整する運用も検討可（現状fakerはRFCOMMベースのためWeb e2eには直接使えない→lib単体テストベクタとしての利用が主）。

---

## 6. リスク・未決事項

1. **実機GATT未検証**（最大リスク。Phase 0で解消）。
2. v2テストに「DRC下位3バイトがoff by 1」のTODO → DRCブロックはpreserve（受信値送り返し）戦略で回避。
3. サービスUUID範囲（±32）外 possibility → `RANGE` 定数拡張で対応可。
4. MACプレフィスリストにP30i OUIが無い場合、フィルタ_mode_で picker に出ない → リスト追加 or 非フィルタ案内。
5. バッテリー0–10スケールの表示丸め（v2は `x/10` 表示。v1 UIはパーセント期待の可能性→変換規則をUI側と揃える）。
6. v1 Webの「最初のprimary service」前提がP30iで成り立つか（Phase 0でサービス一覧を確認）。
7. デプロイCIのメモリ制約は解消済み（wasm-pack導入をプリビルド化済み）。ビルド自体は問題無し。

## 7. 主要参照ファイル

- v2: `lib/src/devices/soundcore/a3959.rs` / `a3959/packets/inbound/state_update.rs` / `a3959/structures/sound_modes.rs` / `a3959/modules/sound_modes/setting_handler.rs` / `device_model.rs` / `soundcore.rs`(RFCOMM UUID) / `connection_backend/*`
- faker: `tools/soundcore-device-faker/devices/a3959.toml`
- v1: `lib/src/device_utils.rs`(GATT UUID/マスク/MACプレフィス) / `lib/src/device_profile.rs` / `lib/src/devices/a3951|a3945/…` / `lib/src/devices/standard/structures/sound_modes_type_two.rs` / `web/wasm/src/web_bluetooth_connection.rs` / `web/src/bluetooth/Device.ts` / `web/src/components/soundModeTypeTwo/*`
