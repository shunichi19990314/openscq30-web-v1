# Phase 0: Soundcore P30i (A3959) 実機検証手順書

目的: P30i が v1 Web版と同じ **BLE GATT プロトコル**で konuşできるかを確定させ、実パケットをテストベクタとして回収する。
所要: 10–20分（実機＋Chrome/Edge搭載PC＋スマートフォン任意）。

## 0. 前提

- v1 Web版の通信経路: GATTサービス `xxxcf5da-0000-1000-8000-00805f9b34fb`（先頭2バイトは機種毎に変化する連続族）
  - 書き込み特性: `00007777-0000-1000-8000-00805f9b34fb`
  - 通知特性:   `00008888-0000-1000-8000-00805f9b34fb`
- 状態要求パケット（wire）: `08 ee 00 00 00 01 01 0a 00 02`
- 期待する状態応答: 約90バイト（コマンドヘッダ込み）、先頭は `09 ff 00 00 01 01 01 …`

## 1. 専用診断ページを使う（推奨）

1. このリポジトリルートでローカルサーバを起動（Web Bluetoothはsecure context必須）:
   ```bash
   python3 -m http.server 8080
   ```
2. Chrome / Edge で `http://localhost:8080/tools/p30i-ble-check.html` を開く。
3. 「1. デバイス選択」→ P30i を選ぶ（イヤホンはケースから出してペアリングモード気味に＝蓋を開けたまま）。
4. 「2. サービス列挙」→ 表が出る。以下を記録:
   - [ ] `…f5da-…` 族サービスの**完全なUUID**（先頭2バイト値が `00fc`〜`013c` の範囲内か確認）
   - [ ] そのサービス内の特性一覧（`7777` / `8888` の存在とプロパティ write / notify）
5. 「3. 通知購読」→ `8888` 特性の notifications を開始。
6. 「4. 状態要求送信」→ `7777` へ状態要求を書き込み。
7. 受信ログに流れてくる hex を**全部コピー**して保存（最低3回分: 接続直後 / ANC切替後 / EQ変更後）。
   - [ ] 応答長（期待: 本体約90バイト）
   - [ ] 先頭バイト列が `09 ff 00 00 01 01 01` で始まるか
8. 任意: イヤホン側でANCモードを切り替えて再度送信し、差分バイトを観察（サウンドモードバイト位置の検証）。

## 2. 代替手段A: Chrome bluetooth-internals

1. `about://bluetooth-internals/#devices` → Scan → P30i を Inspect。
2. Service `…f5da…` を展開、Characteristic `7777` に Write value type: Hex で `08 ee 00 00 00 01 01 0a 00 02` を Write executed。
3. Characteristic `8888` で Subscribe to notifications → 受信 hex を記録。

## 3. 代替手段B: nRF Connect (スマホ)

1. Scan → P30i → Connect。
2. `…f5da…` サービスと `7777`/`8888` 特性の有無をスクリーンショット。
3. `7777` へ上記バイトを書き込み、`8888` の通知 hex をエクスポート。

## 4. 追加で記録したいこと（picker/検出周り）

- [ ] 広告パケットの manufacturerData（先頭3バイトのMACプレフィス）→ `lib/src/device_utils.rs` の `MAC_ADDRESS_PREFIXES` に追加が必要か判定
- [ ] 広告時のデバイス名（"soundcore P30i" 等）
- [ ] 左右分離時の挙動（片側のみケースから出した状態で接続できるか）

## 5. 判定基準

| 結果 | 結論 |
|---|---|
| `…f5da…` サービス + `7777`/`8888` あり & 状態要求に約90B応答 | **Go**: Phase 1実装へ。回収hexを `lib/src/devices/a3959/` の単体テストベクタに追加 |
| サービスはあるが応答なし/別フォーマット | 要追加調査（コマンド差異の逆引き）。Phase 1のparse調整で対応できる範囲を再評価 |
| `…f5da…` サービス無し（GATT非露出） | **No-Go**: v1 WebでのP30iサポートは不可。READMEに制限として記載 |

## 6. 回収データの送付先

- 状態応答hex → `lib/src/devices/a3959/packets/state_update_packet.rs` の `#[cfg(test)]` ベクタ
- サービスUUID先頭値が範囲外 → `lib/src/device_utils.rs` の `RANGE` 定数を拡張
- MACプレフィス → 同上 `MAC_ADDRESS_PREFIXES`
