# nostr-rss
nostrのパブリックチャットにrssの最新の更新を投稿するボットです。
[rust-nostr](https://github.com/rust-nostr/nostr)と[rss](https://github.com/rust-syndication/rss)を使用しています。

## config
- rss
購読するrssの名前(任意)とurlを設定します。実行ファイルと同じディレクトリ配置してください。
```json
[
    {
        "name": "hoge",
        "url": ""
    },
    {
        "name": "huga",
        "url": ""
    }
]
```

- nostr
nostrの秘密鍵とリレー、パブリックチャンネルのイベントidを設定します。実行ファイルと同じディレクトリに配置してください。
```json
{
    "seckey": "nsecXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "channel_id": "",
    "relay": ""
}
```

## ToDo
- [ ] 最新以外の更新も投稿する
- [ ] 監視間隔をサイトごとに変更する
