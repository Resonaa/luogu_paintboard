# luogu_paintboard
洛谷冬日绘板脚本

## 使用教程

### 准备环境
- [Rust](https://www.rust-lang.org/zh-CN/learn/get-started)
- GIMP 或者 Photoshop

### 处理图像
参见 [ouuan / LuoguPaintBoard](https://github.com/ouuan/LuoguPaintBoard#%E5%A4%84%E7%90%86%E5%9B%BE%E5%83%8F)

### 填写配置
- 把处理好的图像放置到 `images` 目录中，重命名为 `#优先级-图像名称(图像左上角x坐标,图像左上角y坐标).bmp`
    - 如：`#1-Test(161,161).bmp` 表示将优先级为 1 的图像 `Test` 绘制到左上角为 (161, 161) 的位置
    - 优先级越大的图像越优先绘制
    - 注意：这里的 x, y 坐标与绘板后端一致，表示 **y 行 x 列**
- 把 `config_example.toml` 复制为 `config.toml`，并填写其中配置项

### 运行脚本
```shell
cargo run --release
```