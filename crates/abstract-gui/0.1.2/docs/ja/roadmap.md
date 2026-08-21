# ロードマップ

## 現在できること

- `.gui` の parse / import / merge / validate
- CLI の一覧・検査コマンド
- HTML scan による page / section / layout / action / index / dialog 抽出
- nav 抽出と過検出抑止
- dialog trigger 抽出と layout への昇格

## 現在の弱点

- 主 nav / 補助 nav の順位付け
- 同一 page を表す複数 URL の統合
- docs taxonomy と site-wide nav の分離
- semantic attribute がない JS modal trigger の検出
- page / layout を超えた dialog 所属推定

## 次の改善候補

- nav ranking の強化
- page alias 正規化
- dialog trigger heuristic の拡張
- docs/site taxonomy 推定の強化
- scan 中間段階の debug 出力
