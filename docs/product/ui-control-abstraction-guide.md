# 主题生成器的 UI 控件抽象笔记

## 1. 这份文档要解决什么问题

你问的其实不是“某个程序里该放什么控件”，而是更底层的问题：

> 一个用户正在编辑某种数据时，UI 应该用什么交互原语来表达这件事？

这里有两个非常重要的结论：

1. **控件首先是“值类型 + 交互语义”的问题，不是视觉样式的问题。**
   - `slider`、`spinbox`、`numeric input` 这些看起来不同，但它们本质上都可能是在编辑一个“有序标量”。
   - `dropdown`、`radio group`、`segmented control` 看起来不同，但本质上都可能是在编辑一个“有限枚举”。

2. **TUI 和 GUI 的真正公共抽象，不是具体 widget，而是“可编辑的数据类型”和“允许的交互能力”。**
   - GUI 可以拖、点、hover、开调色盘、做实时 direct manipulation。
   - TUI 一般只能做：选择、输入、步进、过滤、确认、取消、切换焦点。
   - 所以跨 TUI/GUI 的抽象层，应该建立在“数据语义”和“编辑方式”上，而不是建立在“具体控件长相”上。

---

## 2. 大多数 UI 里的基础控件类型，到底有哪些

这里不按视觉名称分类，而按“用户正在编辑什么值”来分类。

## 2.1 标量值（Scalar）

### 语义

一个有序的单值，通常是数字：

- `0.0 ~ 1.0`
- `0 ~ 100`
- `-12 ~ +12`
- `Hue 0 ~ 360`

### 常见 GUI 表现

- Slider
- Knob / Dial
- SpinBox
- Numeric Input
- Slider + Numeric Input 组合

### 常见 TUI 表现

- 左右键步进
- 数字输入框
- `-` / `+` 按键
- 文本进度条 + 数值输入

### 最适合的抽象

```text
ScalarControl
  value: number
  min/max
  step
  formatter/parser
  optional units
```

### 什么时候用

- 亮度
- 饱和度
- 对比度
- 混合比例
- 透明度
- 大小、间距、时长

---

## 2.2 有限枚举（Enum / Choice）

### 语义

从有限个互斥选项里选一个：

- `Alias / Mix / Adjust / Fixed`
- `Lighten / Darken / Saturate / Desaturate`
- `Horizontal / Vertical`

### 常见 GUI 表现

- Dropdown / Select
- Radio Group
- Segmented Control
- Tabs

### 常见 TUI 表现

- 下拉替代成弹窗列表
- 左右切换
- 单选列表
- Command Palette / Picker

### 最适合的抽象

```text
ChoiceControl<T>
  options: [T]
  selected: T
  labels
```

### 什么时候用

- 规则类型
- 操作类型
- 主题模式
- 导出目标

---

## 2.3 布尔值（Boolean）

### 语义

只有开/关两种状态。

### 常见 GUI 表现

- Switch
- Checkbox

### 常见 TUI 表现

- `[x] / [ ]`
- `On / Off`
- Toggle 行

### 最适合的抽象

```text
BooleanControl
  value: bool
```

### Switch 和 Checkbox 的语义区别

- **Switch** 更适合“系统设置开/关”，强调即时状态。
- **Checkbox** 更适合“是否纳入某集合 / 是否启用某选项”。

换句话说：

- “是否开启自动对比度”更像 switch。
- “是否导出到 VS Code / Neovim / Alacritty”更像 checkbox group。

---

## 2.4 集合成员关系（Set / Multi-select）

### 语义

用户可以从多个候选项里选择多个。

### 常见 GUI 表现

- Checkbox Group
- Multi-select list
- Tag picker

### 常见 TUI 表现

- 带勾选的列表
- 多选弹窗

### 最适合的抽象

```text
SetControl<T>
  all_options: [T]
  selected_set: Set<T>
```

### 什么时候用

- 选择多个 exporter
- 选择启用哪些 preview 面板
- 选择导出哪些 token 组

---

## 2.5 文本值（Text / String）

### 语义

自由输入文本。

### 常见 GUI 表现

- Single-line input
- Textarea
- Search field

### 常见 TUI 表现

- 单行文本输入
- 模式化输入框
- Search/filter 输入

### 最适合的抽象

```text
TextControl
  value: string
  validation
  placeholder
```

### 什么时候用

- Hex color
- 文件路径
- 搜索过滤
- 主题名称

---

## 2.6 引用选择（Reference / Link Selection）

### 语义

不是选择一个原始值，而是选择“引用谁”。

例如：

- `Comment -> TextMuted`
- `Selection -> Accent3`
- `Border -> Surface`

这和普通枚举很像，但又不完全一样，因为候选项可能：

- 分组很多
- 动态变化
- 同时含 semantic token 与 palette slot
- 数量会继续增长

### 常见 GUI 表现

- Dropdown
- Searchable Select
- Picker Dialog
- Tree picker

### 常见 TUI 表现

- Command Palette
- 可过滤的弹窗列表
- 分类 picker

### 最适合的抽象

```text
ReferencePicker<T>
  options: [T]
  groups
  filterable: bool
  selected: T
```

### 什么时候用

- source 选择
- token 引用
- palette 引用

这个在主题生成器里非常重要，**它不应该退化成纯左右键轮询**。

---

## 2.7 颜色值（Color）

### 语义

颜色既是一个值，也经常是一个多维向量：

- RGB
- HSL
- HSV
- OKLCH

### 常见 GUI 表现

- Color picker
- 调色盘
- 色轮
- 渐变条
- Hex input
- RGB/HSL 数值输入

### 常见 TUI 表现

- Hex input
- 颜色 swatch
- H/S/L 三个标量编辑器
- 预设色列表

### 最适合的抽象

```text
ColorControl
  value: Color
  editable_as:
    - literal hex
    - channels
    - preset list
```

### 重要结论

**颜色控件在 GUI 和 TUI 里的“最小公共抽象”不是色轮，而是：**

- 一个颜色值
- 一个文本表示（Hex）
- 若干可编辑通道
- 一个可视化 swatch

---

## 2.8 向量 / 多维连续值（Vector / Multi-axis）

### 语义

用户在编辑一个 2D/3D 的连续值：

- 位置 `(x, y)`
- 方向向量
- 色彩空间中的多通道值

### 常见 GUI 表现

- 2D 平面选择器
- 调色盘面板
- 点选区域

### 常见 TUI 表现

- 拆成多个 scalar
- 预设点列表
- 数字输入

### 最适合的抽象

```text
VectorControl
  dimensions: N
  editable_as_channels
  optional 2D direct manipulation on GUI only
```

### 结论

如果你要跨 TUI/GUI，**多维值不要把“2D 面板拖动”当核心抽象**。  
核心抽象应该是“多通道数值 + 可选的 direct manipulation 表现”。

---

## 2.9 区间 / 成对数值（Range Pair / Interval）

### 语义

一个值不是单点，而是一个区间：

- `[min, max]`
- 阈值上下界

### 常见 GUI 表现

- 双滑块
- 两个输入框

### 常见 TUI 表现

- 两个 scalar 字段
- 先编辑下界，再编辑上界

### 最适合的抽象

```text
RangeControl
  start: number
  end: number
```

---

## 2.10 列表 / 树 / 分层导航（List / Tree / Hierarchy）

### 语义

用户在选择或浏览结构化对象：

- token list
- 分类树
- 文件树

### 常见 GUI 表现

- ListView
- TreeView
- Sidebar

### 常见 TUI 表现

- 列表
- 可折叠树
- breadcrumb + 列表

### 最适合的抽象

```text
SelectionList<T>
  items
  selected
  optional hierarchy
```

---

## 2.11 动作（Action / Command）

### 语义

不是编辑值，而是触发行为。

### 常见 GUI 表现

- Button
- Toolbar action
- Menu item

### 常见 TUI 表现

- 快捷键
- 按钮样式行
- Command Menu

### 最适合的抽象

```text
ActionControl
  id
  label
  enabled/disabled
```

### 什么时候用

- Save
- Load
- Export
- Reset

---

## 2.12 只读反馈（Display / Preview / Status）

### 语义

不是让用户输入，而是让用户理解系统状态。

### 常见 GUI 表现

- Preview
- Status bar
- Toast
- Tooltip
- Swatch

### 常见 TUI 表现

- 状态栏
- 预览面板
- 错误行
- 颜色块

### 最适合的抽象

```text
DisplayPane
StatusMessage
PreviewPane
```

---

## 3. 一个非常重要的原则：控件不是“长相”，而是“语义”

比如：

- `slider`
- `spinbox`
- `numeric input`
- `drag knob`

它们在视觉上是不同控件，但在抽象层上都可能是：

```text
ScalarControl
```

同样：

- `dropdown`
- `radio group`
- `segmented control`
- `command palette`

在抽象层上都可能是：

```text
ChoiceControl / ReferencePicker
```

所以如果你后面要做跨平台 native 渲染，正确的思路不是：

> “我需要统一 slider / dropdown / switch 的实现”

而是：

> “我需要统一 scalar / choice / bool / reference / color / text / action 的语义和状态机”

---

## 4. 你的主题生成器里，涉及到哪些控件类型

下面按你当前 Theme Generator 的数据模型来映射。

## 4.1 ThemeParams

当前参数：

- `background_hue`
- `background_lightness`
- `background_saturation`
- `contrast`
- `accent_hue`
- `accent_saturation`
- `accent_lightness`
- `selection_mix`
- `vibrancy`

### 对应的抽象类型

- `background_hue`, `accent_hue`
  - **ScalarControl**
  - 带单位：角度 `0..360`

- 其余归一化值
  - **ScalarControl**
  - 范围通常是 `0..1`

### GUI 推荐

- Slider + Numeric Input 组合

### TUI 推荐

- 左右键步进
- `Enter` 进入数值输入

### 结论

ThemeParams 基本全是 **标量编辑器**。

---

## 4.2 Token List

当前 token：

- UI tokens
- Syntax tokens
- State tokens

### 对应的抽象类型

- **SelectionList<TokenRole>**
- 带分类层级

### GUI 推荐

- Sidebar / TreeView / Sectioned List

### TUI 推荐

- 分类列表
- 折叠树或按组显示

---

## 4.3 Rule Type

规则类型：

- `Alias`
- `Mix`
- `Adjust`
- `Fixed`

### 对应的抽象类型

- **ChoiceControl<RuleKind>**

### GUI 推荐

- Segmented control
- Dropdown

### TUI 推荐

- 左右切换
- 弹窗单选

---

## 4.4 Source 选择

例如：

- `Alias.source`
- `Mix.a`
- `Mix.b`
- `Adjust.source`

### 对应的抽象类型

- **ReferencePicker<SourceRef>**

### 为什么它不是普通 enum

因为它同时包含：

- Semantic token references
- Palette slot references
- 以后还可能包含 literal / preset / auto sources

而且它需要：

- 分组
- 过滤
- 高亮当前值
- 以后甚至可能需要最近使用项

### GUI 推荐

- Searchable dropdown
- Picker modal
- Tree picker

### TUI 推荐

- 可过滤的弹窗列表
- Command palette

### 结论

这是这个程序里 **最核心的高级控件之一**。

---

## 4.5 Mix Ratio / Adjust Amount

### 对应的抽象类型

- **ScalarControl**

### GUI 推荐

- Slider + Input

### TUI 推荐

- 步进 + 输入

---

## 4.6 Adjust Operation

- `Lighten`
- `Darken`
- `Saturate`
- `Desaturate`

### 对应的抽象类型

- **ChoiceControl<AdjustOp>**

---

## 4.7 Fixed Color

### 对应的抽象类型

- **ColorControl**

### GUI 推荐

- Swatch + Hex Input + Color Dialog
- 可选 HSL/RGB 子面板

### TUI 推荐

- Swatch + Hex Input
- 预设色选择
- H/S/L 数值通道编辑

### 重要建议

如果你后面做 GUI，**不要把 GUI 版 Color Picker 的核心抽象定义成“色轮”**。  
色轮只是 GUI 渲染方式之一。  
核心抽象应该仍然是：

- 颜色值
- Hex 输入
- 通道编辑
- 预设色

---

## 4.8 Save / Load / Export

### 对应的抽象类型

- **ActionControl**
- 如果路径可编辑，还会带一个 **TextControl / PathControl**

---

## 4.9 Preview

### 对应的抽象类型

- **PreviewPane**
- **DisplayPane**

它不是输入控件，但它是整个系统的关键反馈控件。

---

## 5. 你的主题生成器，最终可以归结成哪些“基础交互原语”

如果只看抽象，不看视觉，你这个程序目前核心上只需要这些：

1. **SelectionList**
   - token list

2. **ScalarControl**
   - 所有 theme params
   - mix ratio
   - adjust amount

3. **ChoiceControl**
   - rule kind
   - adjust op

4. **ReferencePicker**
   - source 选择

5. **ColorControl**
   - fixed color

6. **TextControl**
   - hex input
   - filter input
   - 未来路径输入

7. **ActionControl**
   - save/load/export/reset

8. **PreviewPane / Status**
   - sample code
   - swatches
   - status bar

这已经足够覆盖第一阶段的大部分主题编辑需求。

---

## 6. TUI 的局限性到底是什么

这是你后面做抽象时非常关键的一部分。

## 6.1 TUI 通常缺少 pointer-based direct manipulation

GUI 很容易做：

- 拖 slider
- 拖 color wheel
- 点选 2D 色盘
- resize split pane
- drag reorder

TUI 通常不擅长这些。

### 解决方式

把 direct manipulation 降级成：

- 步进
- 输入
- 过滤
- 列表选择
- 模式化编辑

---

## 6.2 TUI 没有 hover 这个核心能力

GUI 可以靠 hover 提供：

- tooltip
- 次级说明
- 渐进暴露

TUI 很难靠 hover。

### 解决方式

改成：

- 当前焦点的底部说明
- 状态栏提示
- Inspector 内联帮助
- modal help

---

## 6.3 TUI 的空间密度和可视化精度有限

GUI 可以同时摆：

- 色轮
- 渐变
- preview
- inspector
- 多标签页

TUI 里如果信息太多，很容易变成密密麻麻的文本墙。

### 解决方式

改成：

- 分页
- 模态弹窗
- 焦点驱动布局
- 分步编辑

---

## 6.4 TUI 的颜色表达能力受终端环境影响

即使终端支持 true color，用户看到的效果也会受：

- terminal theme
- font rendering
- terminal emulator
- 背景透明度

影响。

### 解决方式

- 颜色值一定要有文本表示（hex）
- 预览不要只依赖 swatch
- 重要判断要看 token + preview 结合

---

## 6.5 TUI 不适合复杂的自由形态控件

下面这些 GUI 控件，很难“等价地”搬进 TUI：

- 色轮
- 2D 调色盘
- 贝塞尔曲线编辑器
- 节点图
- 时间轴拖拽
- 自由布局画布

### 解决方式

不要追求外形等价，要追求**语义等价**。

例如：

- 色轮 -> `Hex + H/S/L 通道 + 预设色`
- 节点图 -> `结构化规则表单`
- 拖拽排序 -> `Move Up / Move Down`
- hover tooltip -> `状态栏说明`

---

## 7. GUI 控件到 TUI 控件，最等价的替代方案是什么

下面这个表很重要。

| GUI 常见控件 | 语义抽象 | TUI 最等价方案 |
|---|---|---|
| Slider | ScalarControl | 步进 + 数值输入 + 可视化进度条 |
| SpinBox | ScalarControl | 数字输入 + `- / +` 步进 |
| Dropdown | ChoiceControl | 弹窗列表 / Picker |
| Searchable Select | ReferencePicker | 可过滤 Picker |
| Switch | BooleanControl | Toggle 行 / `[on] [off]` |
| Checkbox | Boolean / Set | `[x] / [ ]` |
| Checkbox Group | SetControl | 多选列表 |
| Radio Group | ChoiceControl | 单选列表 |
| Segmented Control | ChoiceControl | tabs / 左右切换 / 单选列表 |
| Color Dialog | ColorControl | Hex 输入 + 颜色块 + 通道编辑 |
| Color Wheel | ColorControl + VectorControl | H/S/L 数值编辑 + 预设色 |
| Tree View | Hierarchy Selection | 分组列表 / 可折叠树 |
| Drag Reorder | Ordered Collection | Move Up / Move Down |
| Tooltip | Help Display | 状态栏 / 内联帮助 |
| File Dialog | PathControl | 路径输入 + 最近路径列表 |

核心原则：

> **TUI 的“最等价方案”不是复制 GUI 外形，而是保留相同的数据语义和编辑能力。**

---

## 8. 什么东西可以作为 TUI/GUI 的公共抽象交集

如果你后面要做 native GUI，同时保留 TUI，那么最稳妥的“交集”大概就是下面这些：

## 8.1 可以稳定抽象的交集

1. 标量编辑
   - `ScalarControl`

2. 单选枚举
   - `ChoiceControl`

3. 布尔切换
   - `BooleanControl`

4. 多选集合
   - `SetControl`

5. 文本输入
   - `TextControl`

6. 引用选择
   - `ReferencePicker`

7. 颜色值编辑
   - `ColorControl`

8. 列表/树导航
   - `SelectionList`

9. 动作
   - `ActionControl`

10. 只读预览与状态
   - `PreviewPane`
   - `StatusPane`

这些是可以同时在 TUI 和 GUI 上稳定成立的。

## 8.2 不应该作为公共核心抽象的东西

下面这些更适合做“平台增强能力”，而不是基础抽象：

- 鼠标拖拽
- hover
- 2D 色盘拖点
- 色轮
- 节点画布
- drag-and-drop
- 动画手势
- 多指针交互

这些在 GUI 可以有，但不要让业务逻辑依赖它们。

---

## 9. 我建议你如何抽象控件逻辑

如果后面你要让渲染走各平台 native，我建议控制层分成四层。

## 9.1 Domain 层

真正的业务数据：

- `ThemeParams`
- `RuleSet`
- `ResolvedTheme`

这里不关心控件怎么画。

## 9.2 Control Model 层

这一层描述“这个值该怎么编辑”。

可以抽象出类似：

```text
ControlSpec
  id
  label
  kind
  value_type
  current_value
  domain/range
  options
  formatter
  parser
  validator
  capabilities
```

例如：

```text
kind = Scalar
range = 0..1
step = 0.02
formatter = percent
parser = percent_or_unit_float
```

或：

```text
kind = ReferencePicker
groups = [Common Sources, Advanced Palette]
filterable = true
```

## 9.3 Interaction State 层

这一层描述“当前用户正在怎么编辑”：

- 当前焦点
- 是否在输入
- 当前过滤词
- picker 是否打开
- 选中项索引
- 校验错误

这层对 TUI/GUI 都适用。

## 9.4 Renderer 层

这一层才是平台实现：

- TUI renderer
- macOS native renderer
- Windows native renderer
- Linux native renderer

Renderer 只消费：

- `ControlSpec`
- `InteractionState`
- `ViewModel`

它不应该直接控制业务数据规则。

---

## 10. 一个比较实用的跨平台控件抽象清单

如果只保留最有价值、最稳定的公共控件，我建议你最终只维护下面这些抽象类型：

### 1. `ScalarControl`

编辑数值。

### 2. `ChoiceControl`

编辑有限单选枚举。

### 3. `BooleanControl`

编辑 true/false。

### 4. `SetControl`

编辑多选集合。

### 5. `TextControl`

编辑文本。

### 6. `ReferencePicker`

编辑引用关系。

### 7. `ColorControl`

编辑颜色值。

### 8. `SelectionList`

编辑当前选中对象。

### 9. `ActionControl`

执行动作。

### 10. `PreviewPane`

显示效果反馈。

这 10 个已经足够支撑你的主题生成器，也足够支撑很多别的配置型 / 设计型工具。

---

## 11. 回到你的主题生成器，推荐的控件映射

| 领域对象 | 抽象控件类型 | GUI 建议 | TUI 建议 |
|---|---|---|---|
| Token 列表 | SelectionList | Sidebar / Tree | 分组列表 |
| Theme 参数 | ScalarControl | Slider + Input | 步进 + 输入 |
| Rule Type | ChoiceControl | Segmented / Select | 左右切换 / 单选列表 |
| Source | ReferencePicker | Searchable Select | 可过滤 Picker |
| Mix Ratio | ScalarControl | Slider + Input | 步进 + 输入 |
| Adjust Op | ChoiceControl | Dropdown | 单选列表 |
| Adjust Amount | ScalarControl | Slider + Input | 步进 + 输入 |
| Fixed Color | ColorControl | Swatch + Picker + Hex | Swatch + Hex + 预设色 |
| Save/Load/Export | ActionControl | Button / Menu | 快捷键 / 命令 |
| Preview | PreviewPane | 多面板预览 | sample code + mock UI + swatches |

---

## 12. 最后的结论

如果你只记住三件事，我建议记住这三件：

### 结论 1

**不要把控件理解成 slider、dropdown、switch 这些“长相”。**  
正确的抽象是：

- scalar
- choice
- bool
- set
- text
- reference
- color
- action
- selection list
- preview

### 结论 2

**TUI 和 GUI 的公共交集，不在 direct manipulation，而在语义化编辑。**  
也就是说：

- TUI/GUI 可以共享 “我要编辑一个 scalar”
- 但不应该共享 “我要拖一个 slider 轨道”

### 结论 3

**你的主题生成器最值得统一抽象的核心控件只有 5 个：**

1. `ScalarControl`
2. `ChoiceControl`
3. `ReferencePicker`
4. `ColorControl`
5. `SelectionList`

其他大多是它们的组合或辅助。

---

## 13. 对你当前项目的直接建议

如果你真的准备把这个程序做成 TUI + GUI 双实现，那么后面代码结构最好尽量往这个方向靠：

1. 业务数据继续留在 `ThemeParams / RuleSet / Evaluator`
2. 再抽一层 `ControlSpec / ViewModel`
3. TUI 和 GUI 只负责各自渲染
4. 所有输入解析、校验、步进、过滤，都放在共享控制层

这样你未来切到 native GUI 时，不需要重写“控件逻辑”，只需要重写“控件渲染”。

这会比“先做一套 TUI widget，再想办法翻译成 GUI widget”稳定得多。
