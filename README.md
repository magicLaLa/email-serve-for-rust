<div align="center">
<h1>email serve for Rust</h1>
邮件服务（仅支持 SMTP）
</div>

#### 依赖 [lettre](https://github.com/lettre/lettre)
#### 仅支持 发送（send）

#### 使用方式

  ```zsh
    ➜ cargo run # 开发预览
    ➜ cargo build --release # 打包
    ➜ ./target/debug/serve # 运行
  ```

  ![output](./output.png)

#### 接口地址

- `/email/send`

#### TODO

- [ ] Docker 支持
- [ ] Imap 支持