# 卒研
## 作るもの
GraalVMに(というかTruffleに)対応した言語で共通で使える実行視覚化フレームワーク
- コードから実行トレースを生成
- トレースベースで順方向・逆方向の行レベル実行
- 変数の変化を視覚化システムとリンクさせる機構
- あわよくばJavaとかKotlinとかJVMネイティブなやつらも仲間にしてあげたい(特にJavaは教育では重鎮なので……)

## 実行トレースの生成
### Truffleのinspect
- Truffleベースの言語処理系は`--inspect`フラグを渡すことで、`chrome-devtools`プロトコル経由で統一的にデバッグできる  
(TruffleがASTベースらしい[本当?]ので、その辺のおかげで楽にできるのだろうか)
- JVMネイティブな言語は[この方法ではデバッグできない](https://github.com/oracle/graal/blob/master/sulong/docs/DEBUGGING.md)
- `chrome-devtools`のデータをうまいことトレースに落とし込めればいい
- 課題
    - トレースの形式
    - どうやって取る？
        - headless chromeを使って楽にできんか？
        - `chrome-devtools`を読めるライブラリを使って頑張るか……
