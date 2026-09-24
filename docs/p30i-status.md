# P30i (A3959) Web対応 — 状況記録（Status Log）

更新: 2026-09-23（実機検証ラウンド2終了時点）

## 実機で確認済みの事実（Soundcore P30i, FW 01.44）

| 項目 | 値 |
|---|---|
| GATTサービス | `0187f5da-0000-1000-8000-00805f9b34fb`（soundcore族、中心0x011cから+107 → RANGE=256へ拡張済み） |
| 特性 | write `00007777-…` / notify `00008888-…`（v1と同一） |
| 状態応答 | 101バイトwire（body 91B）＝v2/v1パーサと完全一致。テストベクタ化済み |
| シリアル | `39599C2C3139C1A4` → モデル3959 ✓ / MAC `A4:C1:39:31:2C:9C` → プレフィス `A4:C1:39` 追加済み |
| サウンドモード | type3レイアウト（ambient/combined/ambientdup/ncmode/wind/sens/multiscene）✓ |
| 特記 | adaptive感度=255（未報告時の生値）→ スキーマは0–255許可、UIはクランプ表示 |
| EQ | プリセット有効時は band bytes が 0xff 埋め → プリセット名で表示。SET時は10バンドへ0dBパディング |
| フィルタ付きピッカー | 失敗例あり（広告manufacturerDataが旧フォーマットと異なる疑い）→ 診断ページに広告キャプチャ追加済み、データ待ち |

## 適用済み修正（branch: feature/p30i-a3959-web-support）

1. Phase 1: lib移植（a3959モジュール、type3サウンドモード、DeviceModel/registry、テスト）
2. Phase 2: wasmバインディング＋UI（type3セレクション／AdditionalSettings／i18n）
3. RANGE 32→256、MACプレフィス追加、実機ベクタテスト
4. stateスキーマ 0–255 緩和＋UIクランプ、EQ SETパケット10バンドパディング
5. 接続直後のEQ再同期書き込みを同値ガードでスキップ（切断疑い対策）
6. wasmステートリスナーの `.expect` 撤去（JS例外→wasmトラップ→全滅 の連鎖を遮断）

## 未解決（実機/コンソール待ち）

- **接続数秒後の "Device is disconnected."**
  - 監査済み棄却説: 誤parse（全パーサall_consuming）/ JS検証例外のwasm伝播（try/catch済み）/ inbound handler panic（unwrapはtestのみ）/ 自動書き込み（EQガード後はゼロ）
  - 残り候補: ①デバイス又はOS発のGATT切断（ChromeOSペアリングとの取り合い等）②接続直後のstate request応答タイムアウト（再現間欠性）③その他GATTエラー
  - **決定打=Consoleのconsole.error出力**（User側で後日取得予定）
- フィルタ付きピッカーの修正（広告manufacturerDataキャプチャ待ち）
- EQの10バンド完全対応（プリセットテーブルが8バンド分のみの制限。実機EQキャプチャで改善可）
- ~~ボタン設定（Phase 3）~~ → **実装済み**（2026-09-23）:
  - v2 `ActionKind::TwsLowBits` 解明: 1バイト=下ニブルTWS接続時アクション/上ニブル切断時アクション/15=無効
  - 送信はボタン単位パケット `[0x04,0x81] + [order_index, button_id, action_byte]`、変化スロットのみ送信
  - v1 UIの6スロット（単押し/ダブル/長押し×左右）に対応。triple pressはデバイス側保持（UI非編集＝制限事項）
  - 切断時ニブルは既値をpreserve（0xFFスロットは両ニブル同一化）
  - テスト: parseマッピング/送信ニブルpreserve/0xFF無効化 の3件追加

## デプロイ状態

| 対象 | 状態 |
|---|---|
| `feature/p30i-a3959-web-support` | GitHub push済み（最新 a23c9780 以降も追跡） |
| `preview-dist` | CI（build-preview.yml）がfeature push毎にdistを自動公開 |
| Render プレビュー | `openscq30-preview.onrender.com`（branch: preview-dist） |
| Render 診断 | `openscq30-ble-check.onrender.com`（branch: feature/…, publish: tools） |
| 本番 | `main`＝旧dist（A3959未対応）。**GO判定後に preview-dist → main へ反映** |

## 本番反映手順（GO後）

```bash
git fetch origin preview-dist
git push -f origin origin/preview-dist:refs/heads/main   # PAT使用
```
→ 本番Renderサイト（branch: main）が自動デプロイ。以降ボタン設定(Phase 3)へ。
