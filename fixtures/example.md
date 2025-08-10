---
author: 陈小天
title: 和 Gemini 关于第一性原理的对话123c
description: 为了这壶醋，我包了这顿饺子（写了几千行 Rust，做了个工具）
cover: images/02-cover.png
theme: lapis
code: solarized-light
---

## 创新的第一性原理是连接，不是原创

创新的本质是什么？是创造出新的价值。而这种“新”几乎从不是凭空产生的。从信息论和认知科学的底层逻辑看，创新是将已有的、看似不相关的元素，以一种前所未有的方式进行连接和重组。

原创 (Originality) 常被误解为“无中生有”。然而，人类所有的思想和创造，都是建立在已有知识和经验的基础之上。追求绝对的、孤立的原创，就像想拔着自己的头发离开地球一样不可能。

连接 (Connection) 才是创新的真正引擎。无论是史蒂夫·乔布斯将书法艺术与电脑字体连接，还是马斯克将航空技术、软件工程和制造业原理连接，伟大的创新都源于发现和建立新的连接。你的知识储备、经验多样性决定了你有多少个“点”，而你的洞察力决定了你能否把这些点连成有价值的“线”和“面”。

## 领导力的第一性原理是赋能，不是命令

领导力的根本目标是带领一个团队去达成一个共同的目标，并创造出“1+1>2”的集体成果。如何最大化团队的整体产出？

命令 (Command) 是一种单向的权力行使，它能保证任务的执行，但天花板就是领导者本人的认知和能力。下属只是被动的手和脚，无法贡献自己的智慧和主动性。这种模式在工业时代追求效率的流水线上可行，但在知识经济时代则会扼杀活力。

赋能 (Empowerment) 是将权力、资源和信任下放，激发每个团队成员的潜能和主人翁精神。领导者的角色从“发号施令者”转变为“服务者”和“环境营造者”。通过赋能，团队的智慧被汇聚，适应性和创造力大大增强，从而能达成远超领导者个人能力上限的成就。

![first-principle](images/first-principle.jpg)

```rust
fn main() {
    println!("Hello, world!");
}
```

## 解决问题的第一性原理是定义问题，不是寻找答案

解决问题的本质是消除现状与期望之间的差距。

寻找答案是在已经明确“差距”是什么之后采取的行动。但如果我们对问题的定义本身就是错的，那么我们找到的每一个答案，无论多么精妙，都只是在一个错误的战场上打赢了一场无关紧要的战役。

定义问题是整个过程中最关键、也最需要智慧的一步。它要求我们深入探究表象之下真正的根本原因（Root Cause）。爱因斯坦曾说：“如果我有一个小时来拯救世界，我会花55分钟来定义问题，只花5分钟来寻找解决方案。” 精准地定义了问题，解决方案往往会自然浮现。

![ai](images/ai-comm.jpg)

## 谈判的第一性原理是共赢，不是征服

谈判的根本目的是通过与他人的协商，达成一个比不协商更好的结果。这是一个寻求合作以创造增量的过程，而非零和博弈。

征服 (压倒对方) 将谈判视为一场战争，目标是让对方输，自己赢。这种心态可能让你在单次博弈中获得短期利益，但会损害长期关系，甚至导致谈判破裂，最终双方都得不到任何好处。

共赢 (寻找共同利益) 是将双方视为解决共同问题的合作伙伴。它要求我们从“立场”之争，转向挖掘双方背后真正的“利益”所在，然后通过创造性地“做大蛋糕”来满足双方的核心需求。只有建立在共赢基础上的协议，才是最稳固、最能被忠实执行的。

于是，看似简约的 API 下，隐藏着不少设计考量：

1. 上传图片时，要小心别重复了。公众号接口不能按名字找，只能一个个翻看。我只翻看最近的二十张，给每张图片贴上内容的 blake3 哈希值标签，这样只要内容相同，且在这二十张里，就不会重复上传。
2. 上传图文草稿时，也得防止重复。我用标题来判断是否已经上传过。如果发现已经有了，就更新它。
3. css 主题我用了 wenyan-mcp 里的样式。这个 mcp 是专门用来上传公众号文章的，但不知为何它总是罢工。
4. 公众号对 css 有限制，只能内嵌。我就想了些办法（使用 css-inline 和 regex），把全局 css 变成内嵌。

### Nested Lists

1. Parent item 1
   - Nested bullet 1
   - Nested bullet 2
2. Parent item 2
   1. Nested number 1
   2. Nested number 2

## Inline Code

Here's some inline code: `const x = 42;` in the middle of text.

## Simple Code Block

```rust
fn main() {
    println!("Hello, world!");
}
```

## Code Block with Syntax Highlighting

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // This is a comment
    let config = Config::from_env()?;
    let client = WeChatClient::new(&config);

    println!("Client initialized!");
    Ok(())
}
```

```rust
async fn submission_loop(
    session_id: Uuid,
    config: Arc<Config>,
    auth: Option<CodexAuth>,
    rx_sub: Receiver<Submission>,
    tx_event: Sender<Event>,
    ctrl_c: Arc<Notify>,
) {
    let mut sess: Option<Arc<Session>> = None;

    loop {
        let sub = tokio::select! {
            res = rx_sub.recv() => match res {
                Ok(sub) => sub,
                Err(_) => break,
            },
            _ = ctrl_c.notified() => {
                // 优雅处理中断
                if let Some(sess) = sess.as_ref() {
                    sess.abort();
                }
                continue;
            },
        };

        // 处理提交...
    }
}
```

## JavaScript Code with Special Characters

```javascript
const regex = /<code>([^<]*)</code>/;
const html = "<pre><code>test</code></pre>";
console.log("Regex test:", regex.test(html));
```

## Multiple Code Blocks

First block:

```python
def hello():
    print("Hello from Python")
```

Second block:

```go
func main() {
    fmt.Println("Hello from Go")
}
```

## Code with Indentation

```yaml
server:
  host: localhost
  port: 8080
  features:
    - authentication
    - logging
    - monitoring
```
