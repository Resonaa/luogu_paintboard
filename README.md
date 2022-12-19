# luogu_paintboard
洛谷冬日绘板脚本

## 使用教程

### 准备环境
- [Rust](https://www.rust-lang.org/zh-CN/learn/get-started)
- GIMP 或者 Photoshop

### 处理图像

#### BMP 图像
参见 [ouuan / LuoguPaintBoard](https://github.com/ouuan/LuoguPaintBoard#%E5%A4%84%E7%90%86%E5%9B%BE%E5%83%8F)

#### 字符画
字符画功能可以很方便地绘制微型字符画，只需按照绘板后端格式填写颜色，以 txt 为后缀保存即可

如：
```
0100010
0101110
0100010
0101010
0100010
```

可以绘制白底黑字的“161”

注意：字符画为正立方向（符合正常人的认知规律），与绘板后端格式相反

### 填写配置
- 把处理好的图像放置到 `images` 目录中，重命名为 `#优先级-图像名称(图像左上角x坐标,图像左上角y坐标).bmp/txt`
    - 如：`#1-Test(161,161).bmp` 表示将优先级为 1 的 BMP 图像 `Test` 绘制到左上角为 (161, 161) 的位置
    - 优先级越小的图像越优先绘制
    - 注意：这里的 x, y 坐标与绘板后端一致，表示 **y 行 x 列**（与正常人的认知规律相反）
- 把 `config_example.toml` 复制为 `config.toml`，并填写其中配置项

### 运行脚本
```shell
cargo run --release
```