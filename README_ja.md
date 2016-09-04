# Tripcode
[![ビルド状況](https://travis-ci.org/huton/tripcode-rs.svg?branch=master)](https://travis-ci.org/huton/tripcode-rs)
[![最新バージョン](http://meritbadge.herokuapp.com/tripcode)](https://crates.io/crates/tripcode)

[ドキュメント（英語）](https://docs.rs/tripcode/)

[README in English](README.md)

電子掲示板で使われているトリップをRustで扱うためのライブラリです。

## 対応状況

このクレートは以下の形式のトリップに対応しています。

* 2ちゃんねる（.net）のトリップ
    * 10桁トリップ
    * 生キートリップ
    * 12桁トリップ
* 2ちゃんねる（.sc）のトリップ
    * 15桁トリップ
    * カタカナトリップ
* 4chanの（secureでない）トリップコード

## 使い方

このクレートを使用するにはあなたのプロジェクトの`Cargo.toml`に以下を追加します。

```toml
[dependencies]
tripcode = "0.1"
```

そしてクレートのルートに以下の行を追加します。

```rust
extern crate tripcode;
```

## 例

```rust
let mut tripcode;

// 2ちゃんねる（Monazilla）のトリップ
tripcode = Mona::generate(&"7 bytes");
assert_eq!("W/RvZlE2K.", &tripcode);
tripcode = Mona::generate(&"twelve bytes");
assert_eq!("t+lnR7LBqNQY", &tripcode);
tripcode = Mona::generate(&"#1145145554560721..");
assert_eq!("14cvFmVHg2", &tripcode);

// 10桁トリップ
tripcode = Mona10::generate(&"password longer than 12 bytes");
assert_eq!("ozOtJW9BFA", &tripcode);

// 生キートリップ。`try_generate()`は失敗時に`None`を返す
tripcode = MonaRaw::try_generate(&"#0123456789ABCDEF./").unwrap();
assert_eq!(&"IP9Lda5FPc", &tripcode);

// 12桁トリップ
tripcode = Mona12::generate(&"<12 bytes");
assert_eq!("/9L00Vb1PBcb", &tripcode);

// 4chanのトリップ
tripcode = Fourchan::generate(&"password");
assert_eq!("ozOtJW9BFA", &tripcode);
```

## `tripcode`コマンド

このクレートは、トリップを生成するための簡単なコマンドを提供します。

コマンドをインストールするには、シェルで以下を実行します。

```bash
cargo install tripcode
```

`tripcode`コマンドはデフォルトで4chanのトリップを生成します。
2ちゃんねるのトリップを生成するには`--type=2ch`オプションを使用します。

`tripcode`コマンドはパスワードを引数に取ります。

```bash
$ tripcode --type=2ch a b c
ZnBI2EKkq.
taAZ7oPCCM
wG1CV58ydQ
```

また、パスワードは標準入力から改行区切りで与えることもできます。

```bash
$ echo 'd\ne\nf' | tripcode --type=2ch -f
taZqHR8ods
xKvzozvsSk
bb6OCCHf8E
```

このコマンドは非UTF-8のエンコーディングでも動作します。

```bash
$ echo トリップ | iconv -t sjis | tripcode --type=2ch -f
XSSH/ryx32
```
