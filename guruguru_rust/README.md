# サンプル：ぐるぐるRust

## 概要

ぐるぐる回るRustのロゴを M5Stack のLCDに表示するサンプルです。

## ビルド方法

* 以下の環境変数を設定
    * `$WORKDIR` は、rustcをコンパイルしたときのワーキングディレクトリを表す
        * `$WORKDIR/rust` にrustcのソース
        * `$WORKDIR/build/llvm-xtensa` にビルドしたXtensa対応LLVM
        * `$HOME/esp/esp-idf` にESP-IDF
        * `$HOME/esp/xtensa-esp32-elf` にXtensaのgccツールチェイン

    ```bash
    export LIBCLANG_PATH=$WORKDIR/build/llvm-xtensa/lib
    export XARGO_RUST_SRC=$WORKDIR/rust/src
    export XTENSA_TOOLCHAIN_ROOT=$HOME/esp/xtensa-esp32-elf
    export IDF_PATH=$HOME/esp/esp-idf
    export PATH=$XTENSA_TOOLCHAIN_ROOT/bin:$PATH
    ```

* 埋め込み用画像を生成

    * ImageMagickが無ければ入れる
        ```bash
        sudo apt-get install -y imagemagick
        ```

    ```
    pushd assets
    make
    popd
    ```


* ESP-IDFのプロジェクトとしてビルド

    ```
    make
    ```

