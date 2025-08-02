# Instructions

@agent-product-spec-architect @agent-rust-backend-expert 请阅读 wechat go sdk 的代码(在 ./go 下),
专注微信公众号如何上传图片和上传图文草稿的部分,将这部分代码用 Rust 实现. 请先认真思考,在 ./specs/0001-rust-sdk.md
下创建中文的设计文档.注意第一版就只处理上传图片和上传图文草稿的部分. sdk 要足够简单: 初始化微信公众号实例 wx,然后
wx.upload("./abc.md", "theme1"). 其中, theme1 是某个 css 样式. 在 upload 流程里,先查看 markdown 里的 image
url,将其分别上传,然后替换成上传后的url,然后再根据 theme 把 markdown render 成 html,上传成图文草稿.

@agent-rust-backend-expert  请根据 @fixtures/example.md 调整 sdk 的实现 - 如果没有特定制定 cover / title
等信息,那么 md 文件中的 frontmatter 中应该包含相应的信息,如果既没有通过 options,frontmatter 里又没有带
cover,则报错. 请确保 cargo clippy & cargo test 全部通过,之后 commit 代码
