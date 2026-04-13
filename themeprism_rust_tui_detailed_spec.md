# ThemePrism for Everyone – Rust + TUI Engineering Spec

## 1. 文档目标

这份文档定义一个 **面向普通用户但仍具备高级可扩展性** 的 ThemePrism 设计方案，第一阶段使用 **Rust + TUI** 实现。

本文档目标是让实现者能够直接开始搭建：

- Rust 项目结构
- 核心数据模型
- 规则系统
- TUI 交互
- 预览系统
- 导出系统
- 分阶段里程碑

这个工具不是一个通用 node graph 平台，也不是一个只会输出固定模板的 theme preset 机器。  
它的定位是：

> 一个以 **参数系统 + 语义 token + 受限规则系统 + 多目标导出** 为核心的 ThemePrism。

---

## 2. 产品定位与核心原则

### 2.1 产品定位

产品目标用户分两类：

1. **普通用户 / 非开发者**
   - 通过 UI 调整主题外观
   - 不需要理解 DSL / graph / 表达式树
   - 重点是“结果”和“感觉”

2. **高级用户 / geek / 开发者**
   - 希望精确控制 token 生成逻辑
   - 希望覆盖局部规则
   - 希望导出到具体工程配置

因此系统必须做到：

- 默认易用
- 高级可扩展
- 不强迫用户理解底层规则语言

---

### 2.2 为什么不做 node graph

完整 node graph 会让产品边界滑向“通用可视化编程系统”，带来以下问题：

- UI 复杂度暴涨
- 学习成本高
- 偏离“主题设计”这个核心场景
- 需要额外解决 graph 调试、版本管理、循环依赖、节点库维护等问题

本产品不暴露 node graph UI。

但注意：

> **内部实现可以使用 DAG / 依赖解析 / 拓扑排序思想。**

也就是说：

- **内部可以是图**
- **外部不表现为图**

---

### 2.3 为什么不能全部写死

如果所有 token 生成规则都写死在代码里，会导致：

- 可扩展性差
- 很多主题风格无法表达
- 用户无法进行局部微调
- 最终变成“有限 preset 工具”

因此必须引入：

> **受限规则系统（Restricted Rule System）**

它比写死灵活，但比通用 graph 更聚焦。

---

### 2.4 核心设计原则

#### 原则 1：亮度优先于色相
Theme 的可读性主要依赖亮度对比，而不是 hue 本身。  
生成系统应优先建立稳定的 background / foreground / accent 亮度结构。

#### 原则 2：语义优先于颜色名
系统内部应使用语义 token，而不是直接围绕“红、蓝、绿”组织逻辑。

例如：

- `error`
- `warning`
- `selection`
- `keyword`
- `comment`

而不是：

- `red`
- `blue`

#### 原则 3：UI 编辑的是“意图”，不是表达式
普通用户不应该直接面对：

```text
selection = mix(accent_3, bg_1, 0.35)
```

而应看到：

- 主颜色：Accent Blue
- 背景颜色：Surface
- 混合比例：35%

#### 原则 4：规则系统必须结构化
内部真相来源不应是 DSL 字符串，而应是结构化 Rule Model。

#### 原则 5：预览必须与导出一致
Preview 所见即所得。  
UI 显示的颜色与导出文件中的颜色必须一致。

#### 原则 6：先做受限可用，再做完全自由
第一阶段优先实现：

- 有限 Rule 类型
- 有限 Source 类型
- 有限 token 集
- 有限 exporter

而不是追求完整设计系统平台。

---

## 3. 总体架构

系统采用如下流水线：

```text
ThemeParams
    ↓
Palette Generator
    ↓
RuleSet + Evaluator
    ↓
Resolved Semantic Tokens
    ↓
Preview Renderer
    ↓
Export Adapters
    ↓
Theme Files
```

各层职责：

### 3.1 ThemeParams
全局控制参数，例如：

- background hue
- background lightness
- contrast
- accent hue
- accent saturation
- vibrancy

### 3.2 Palette Generator
由参数生成基础调色板（palette slots）：

- bg_0 bg_1 bg_2
- fg_0 fg_1 fg_2
- accent_0 ... accent_5

### 3.3 RuleSet + Evaluator
将基础 palette 解析为语义 token：

- comment
- keyword
- selection
- border
- error
- ...

### 3.4 Preview Renderer
将最终 token 渲染到：

- sample code
- palette swatches
- token list
- mock UI preview

### 3.5 Export Adapters
将语义 token 映射为具体工具的配置字段，例如：

- Alacritty
- Neovim
- VS Code

---

## 4. 用户层级与 UI 模式

系统应支持三个抽象层，但第一阶段只需实现 Advanced。

### 4.1 Beginner 模式（未来）
适合普通用户，仅暴露高层参数：

- 背景深浅
- 整体对比度
- 强调色
- 主题冷暖
- 鲜艳度

不直接显示 token 和规则。

### 4.2 Advanced 模式（第一阶段主模式）
显示：

- token 列表
- token rule inspector
- palette
- preview

这是第一阶段的主要交互模式。

### 4.3 Expert 模式（未来）
显示：

- 规则的原始结构化内容
- palette slots
- raw mapping
- exporter override

---

## 5. Theme 参数系统

第一阶段建议的 MVP 参数如下。

### 5.1 Core ThemeParams

- `background_hue`
- `background_lightness`
- `background_saturation`
- `contrast`
- `accent_hue`
- `accent_saturation`
- `accent_lightness`
- `selection_mix`
- `vibrancy`

这些参数足以生成一个可用主题系统。

---

## 6. Palette 系统

Palette 是参数系统输出的基础颜色槽位，不带直接用户语义。

### 6.1 基础 palette slots

- `bg_0`
- `bg_1`
- `bg_2`
- `fg_0`
- `fg_1`
- `fg_2`
- `accent_0`
- `accent_1`
- `accent_2`
- `accent_3`
- `accent_4`
- `accent_5`

### 6.2 Palette 的意义

#### Background ladder
- `bg_0`: 主背景
- `bg_1`: 次背景 / panel / surface
- `bg_2`: 更高亮的 surface / border base

#### Foreground ladder
- `fg_0`: muted text / comment
- `fg_1`: normal text
- `fg_2`: strong text / cursor / emphasis

#### Accent family
- `accent_0 ... accent_5`: 用于 syntax / state / emphasis

---

## 7. 语义 Token Schema

第一阶段建议支持以下 token。

### 7.1 UI Tokens

- `Background`
- `Surface`
- `SurfaceAlt`
- `Text`
- `TextMuted`
- `Border`
- `Selection`
- `Cursor`

### 7.2 Syntax Tokens

- `Comment`
- `Keyword`
- `String`
- `Number`
- `Type`
- `Function`
- `Variable`

### 7.3 State Tokens

- `Error`
- `Warning`
- `Info`
- `Hint`
- `Success`

这套 token 足以覆盖 terminal/editor/TUI preview 的 MVP。

---

## 8. Rule System 设计

### 8.1 Rule System 的定位

规则系统用于回答：

> 某个 token role 的最终颜色是如何生成的？

例如：

- `Keyword` 是否直接来自 `accent_4`
- `Selection` 是否来自 `accent_3` 与 `Surface` 的混合
- `Border` 是否来自 `Surface` 的提亮版本

### 8.2 为什么 UI 不直接暴露 DSL

普通用户不应看到：

```text
selection = mix(accent_3, bg_1, 0.35)
```

而应看到表单型 inspector。

因此内部需要：

- 结构化 Rule Model
- evaluator
- 可选 DSL / TOML 导出

---

### 8.3 Rule Archetypes

第一阶段推荐支持以下 5 种 Rule。

#### 1. Auto
系统自动生成颜色。

适合未来扩展：
- border
- cursor
- selection
- line highlight

第一阶段可以暂缓实现。

#### 2. Alias
直接引用某个 source。

例如：
- `Comment -> TextMuted`
- `Keyword -> Accent4`

#### 3. Mix
由两个 source 按比例混合。

例如：
- `Selection -> mix(Accent3, Surface, 0.35)`

#### 4. Adjust
基于一个 source 做单步调整。

例如：
- `Border -> lighten(Surface, 0.08)`
- `Comment -> desaturate(Text, 0.2)`

#### 5. Fixed
直接给定一个具体颜色。

例如：
- `Keyword -> #c586c0`

---

### 8.4 第一阶段建议
为了降低复杂度，第一阶段建议实现：

- `Alias`
- `Mix`
- `Adjust`
- `Fixed`

`Auto` 可以等第二阶段再做。

---

## 9. Source Model 设计

Rule 会引用颜色来源，因此需要统一 Source 模型。

### 9.1 Source 类型

推荐支持：

#### Semantic Token Source
引用某个已解析 token。

例如：
- `Text`
- `TextMuted`
- `Surface`
- `Error`

#### Palette Slot Source
引用某个基础 palette slot。

例如：
- `Bg0`
- `Fg0`
- `Accent3`

#### Literal Color
固定颜色值。

例如：
- `#89b4fa`

---

### 9.2 UI 中的 Source 分层

在 UI 中，source 不应全部平铺显示。

应分为：

#### Common Sources
- Background
- Surface
- SurfaceAlt
- Text
- TextMuted
- Accent-like roles
- Error / Warning / Success 等

#### Advanced Sources
- bg_0
- bg_1
- bg_2
- fg_0
- fg_1
- accent_0 ... accent_5

普通用户默认看 Common，高级用户可展开 Advanced。

---

## 10. Adjust Operation 设计

第一阶段建议支持以下操作：

- `Lighten`
- `Darken`
- `Saturate`
- `Desaturate`

第二阶段可扩展：
- `RotateHue`
- `ShiftTowardAccent`
- `ShiftTowardBackground`

---

## 11. Rust 类型定义框架

下面给出建议的数据结构骨架。实现时可直接用这些类型起步。

```rust
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}
```

---

### 11.1 Theme parameters

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeParams {
    pub background_hue: f32,
    pub background_lightness: f32,
    pub background_saturation: f32,

    pub contrast: f32,

    pub accent_hue: f32,
    pub accent_saturation: f32,
    pub accent_lightness: f32,

    pub selection_mix: f32,
    pub vibrancy: f32,
}
```

---

### 11.2 PaletteSlot

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PaletteSlot {
    Bg0,
    Bg1,
    Bg2,
    Fg0,
    Fg1,
    Fg2,
    Accent0,
    Accent1,
    Accent2,
    Accent3,
    Accent4,
    Accent5,
}
```

---

### 11.3 TokenRole

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenRole {
    // UI
    Background,
    Surface,
    SurfaceAlt,
    Text,
    TextMuted,
    Border,
    Selection,
    Cursor,

    // Syntax
    Comment,
    Keyword,
    String,
    Number,
    Type,
    Function,
    Variable,

    // State
    Error,
    Warning,
    Info,
    Hint,
    Success,
}
```

---

### 11.4 SourceRef

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SourceRef {
    Token(TokenRole),
    Palette(PaletteSlot),
    Literal(Color),
}
```

---

### 11.5 AdjustOp

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustOp {
    Lighten,
    Darken,
    Saturate,
    Desaturate,
}
```

---

### 11.6 AutoStrategy（第二阶段预留）

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoStrategy {
    MatchRole,
    ContrastDerived,
    AccentDerived,
    SurfaceDerived,
    StateSemantic,
}
```

---

### 11.7 AutoOptions（第二阶段预留）

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct AutoOptions {
    pub intensity: f32,
    pub keep_contrast: bool,
    pub follow_global_vibrancy: bool,
}
```

---

### 11.8 Rule

第一阶段推荐的 Rule 定义：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    Alias {
        source: SourceRef,
    },

    Mix {
        a: SourceRef,
        b: SourceRef,
        ratio: f32,
    },

    Adjust {
        source: SourceRef,
        op: AdjustOp,
        amount: f32,
    },

    Fixed {
        color: Color,
    },

    // 第二阶段再启用
    Auto {
        strategy: AutoStrategy,
        options: AutoOptions,
    },
}
```

如果第一阶段想更简单，可以直接先去掉 `Auto`。

---

### 11.9 RuleSet

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSet {
    pub rules: BTreeMap<TokenRole, Rule>,
}
```

---

### 11.10 Palette / ResolvedTheme

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub slots: BTreeMap<PaletteSlot, Color>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedTheme {
    pub palette: Palette,
    pub tokens: BTreeMap<TokenRole, Color>,
}
```

---

### 11.11 Evaluation error

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    CycleDetected(Vec<TokenRole>),
    MissingRule(TokenRole),
    MissingPaletteSlot(PaletteSlot),
    InvalidRatio(f32),
    InvalidAmount(f32),
    InvalidSource(String),
}
```

---

## 12. Rule Evaluator 设计

### 12.1 输入与输出

输入：
- `ThemeParams`
- `Palette`
- `RuleSet`

输出：
- `ResolvedTheme`

### 12.2 Evaluator 职责

- resolve palette slot colors
- resolve token colors from rules
- 支持 token → token 引用
- 支持 palette → token 引用
- 检测循环依赖
- 做 memoization
- 返回稳定的最终颜色集

---

### 12.3 Evaluator 行为规范

#### Alias
直接 resolve source。

#### Mix
先 resolve `a` 和 `b`，再按比例混合。

#### Adjust
先 resolve source，再执行单步颜色变换。

#### Fixed
直接返回字面颜色。

#### Auto
由系统自动推导，第一阶段可不启用。

---

### 12.4 依赖解析

如果 token 可以引用别的 token，例如：

- `Comment -> TextMuted`
- `TextMuted -> Fg0`

那么 evaluator 内部需要递归解析。

建议采用：

- DFS resolve
- visited stack 检测循环
- resolved cache 存储结果

这样内部本质上是一个 DAG evaluator。

---

## 13. 默认 RuleSet

系统应内置一套默认规则，使项目启动后立即可用。

推荐默认规则如下：

```text
Background -> Alias(bg_0)
Surface    -> Alias(bg_1)
SurfaceAlt -> Alias(bg_2)
Text       -> Alias(fg_1)
TextMuted  -> Alias(fg_0)
Border     -> Adjust(source=Surface, op=Lighten, amount=0.08)
Selection  -> Mix(a=Accent3, b=Surface, ratio=0.35)
Cursor     -> Alias(fg_2)

Comment    -> Alias(TextMuted)
Keyword    -> Alias(Accent4)
String     -> Alias(Accent1)
Number     -> Alias(Accent2)
Type       -> Alias(Accent3)
Function   -> Alias(Accent5)
Variable   -> Alias(Text)

Error      -> Alias(Accent0)
Warning    -> Alias(Accent2)
Info       -> Alias(Accent3)
Hint       -> Alias(Accent5)
Success    -> Alias(Accent1)
```

这组规则非常适合作为 MVP 起点。

---

## 14. Palette Generator 设计

第一阶段不要求复杂色彩空间，HSL 即可验证系统链路。

### 14.1 输入

使用 `ThemeParams` 作为输入：

- background hue/lightness/saturation
- accent hue/saturation/lightness
- contrast
- vibrancy

### 14.2 输出

生成：

- `bg_0..bg_2`
- `fg_0..fg_2`
- `accent_0..accent_5`

### 14.3 设计目标

- 背景层次稳定
- 前景与背景有足够对比
- accent family 在 hue 上拉开间隔
- 保持低背景饱和度与中等 accent 饱和度

### 14.4 第二阶段升级

第二阶段可考虑引入：

- OKLCH
- 感知亮度 spacing
- hue-specific lightness compensation

---

## 15. TUI 交互设计

第一阶段目标平台是 TUI。推荐使用：

- `ratatui`
- `crossterm`

### 15.1 TUI 总体布局

```text
┌──────────────┬──────────────────────┬──────────────────────┐
│ Token List   │ Preview              │ Inspector            │
│              │                      │                      │
│ UI           │ sample code          │ selected token rule  │
│ Syntax       │ palette              │ rule editor          │
│ State        │ token swatches       │                      │
├──────────────┴──────────────────────┴──────────────────────┤
│ Status / help / export info                                │
└────────────────────────────────────────────────────────────┘
```

### 15.2 三个主区域

#### 左侧：Token List
显示 token 分类树，并支持选择当前 token。

#### 中间：Preview
显示 sample code、palette、token swatches。

#### 右侧：Inspector
显示当前选中 token 的规则编辑 UI。

#### 底部：Status Bar
显示：
- 按键提示
- 当前模式
- 导出状态
- 错误消息

---

## 16. Inspector 设计

Inspector 是整个产品的核心 UI，不是 DSL 编辑器。

### 16.1 Inspector 顶部显示

- 当前 token 名称
- 当前 token 颜色 swatch
- 当前 hex 值
- 当前 rule type

### 16.2 Rule Type 切换

第一阶段提供：

- Alias
- Mix
- Adjust
- Fixed

用户切换 rule type 后，下面显示对应表单。

---

### 16.3 Alias Inspector

字段：

- `Source`

UI 形式：

```text
Rule Type: Alias
Source: [Muted Text ▼]
```

---

### 16.4 Mix Inspector

字段：

- `Color A`
- `Color B`
- `Blend Ratio`

UI 形式：

```text
Rule Type: Mix
Color A: [Accent 3 ▼]
Color B: [Surface ▼]
Blend : [------●---] 35%
```

---

### 16.5 Adjust Inspector

字段：

- `Source`
- `Operation`
- `Amount`

UI 形式：

```text
Rule Type: Adjust
Source   : [Surface ▼]
Operation: [Lighten ▼]
Amount   : [---●------] 8%
```

---

### 16.6 Fixed Inspector

字段：

- `Hex Color`

第一阶段 TUI 不需要完整 color wheel，仅支持：
- hex 输入
- 常用颜色选择
- 从 palette 复制后微调（可选）

UI 形式：

```text
Rule Type: Fixed
Color: #c586c0
```

---

## 17. Preview 设计

Preview 是 ThemePrism 是否好用的关键。

### 17.1 必须包含的部分

#### Sample Code Preview
至少显示一段带 syntax token 的代码。

建议可用 Rust 或 C++ 示例。

#### Palette Swatches
显示：
- bg_0..bg_2
- fg_0..fg_2
- accent family

#### Token Swatches
显示每个 semantic token 的最终颜色。

---

### 17.2 Sample Code 示例目标

应至少覆盖：
- keyword
- function
- string
- number
- variable
- comment

---

## 18. Export 系统设计

### 18.1 核心思想

导出器不直接依赖 ThemeParams，而依赖：

- `ResolvedTheme`

也就是说：

```text
ThemeParams -> Palette -> RuleSet -> ResolvedTheme -> Exporter
```

这保证：

- 预览与导出一致
- exporter 与生成逻辑解耦

---

### 18.2 第一阶段 exporter

建议只实现一个 exporter：

- `Alacritty`

原因：
- 格式简单
- 验证主题文件输出链路容易
- terminal 颜色槽位有限

### 18.3 第二阶段 exporter

- Neovim
- VS Code

---

### 18.4 Exporter trait 建议

```rust
pub trait Exporter {
    fn name(&self) -> &'static str;
    fn export(&self, theme: &ResolvedTheme) -> Result<String, ExportError>;
}
```

---

### 18.5 ExportError

```rust
#[derive(Debug)]
pub enum ExportError {
    MissingToken(String),
    SerializeError(String),
}
```

---

## 19. 配置文件格式建议

项目保存格式不建议以 DSL 字符串为主，而建议使用结构化 TOML / JSON。

推荐 TOML。

示例：

```toml
[params]
background_hue = 220.0
background_lightness = 0.12
background_saturation = 0.08
contrast = 0.85
accent_hue = 185.0
accent_saturation = 0.55
accent_lightness = 0.62
selection_mix = 0.35
vibrancy = 0.50

[rules.Text]
type = "alias"
source = "Fg1"

[rules.TextMuted]
type = "alias"
source = "Fg0"

[rules.Selection]
type = "mix"
a = "Accent3"
b = "Surface"
ratio = 0.35

[rules.Border]
type = "adjust"
source = "Surface"
op = "Lighten"
amount = 0.08
```

Rust 里可用 `serde` 读写。

---

## 20. Rust 项目结构建议

推荐目录结构：

```text
src/
  main.rs
  app.rs
  event.rs
  ui.rs
  state.rs

  color.rs
  params.rs
  palette.rs
  tokens.rs
  rules.rs
  evaluator.rs
  preview.rs

  export/
    mod.rs
    alacritty.rs

  persistence/
    mod.rs
    project_file.rs
```

### 20.1 各模块职责

#### `main.rs`
程序入口，初始化 terminal，进入 event loop。

#### `app.rs`
应用控制器，处理事件与状态更新。

#### `state.rs`
保存当前 UI 状态和业务状态。

#### `color.rs`
颜色类型、混色、lighten/darken、hex 转换等。

#### `params.rs`
`ThemeParams` 及参数默认值。

#### `palette.rs`
Palette slot 生成逻辑。

#### `tokens.rs`
`TokenRole`、`PaletteSlot` 等定义。

#### `rules.rs`
`Rule`、`SourceRef`、`AdjustOp`、`RuleSet` 定义。

#### `evaluator.rs`
Rule evaluator / token resolution / cycle detection。

#### `preview.rs`
sample code tokenization 和 preview 数据生成。

#### `ui.rs`
ratatui 布局与绘制逻辑。

#### `export/`
theme file exporters。

#### `persistence/`
保存与加载项目配置。

---

## 21. 阶段性目标（Milestones）

下面给出建议的阶段目标，便于 Codex 分阶段开发。

---

### Milestone 1：基础骨架与参数预览

目标：
- TUI 启动
- 三栏布局
- 参数结构定义
- palette 生成
- palette swatch 显示

完成标准：
- 用户可以启动程序
- 修改全局参数
- 实时看到基础 palette 变化

范围：
- `ThemeParams`
- `PaletteSlot`
- `Palette`
- `color.rs`
- `palette.rs`
- 基础 `ui.rs`

---

### Milestone 2：Token 与 Rule 系统

目标：
- 定义 `TokenRole`
- 定义 `Rule` / `SourceRef` / `RuleSet`
- 实现默认 RuleSet
- 实现 evaluator
- 支持 token swatch 预览

完成标准：
- 可以从 palette 解析出 semantic tokens
- 支持 Alias / Mix / Adjust / Fixed
- 能检测循环依赖

范围：
- `tokens.rs`
- `rules.rs`
- `evaluator.rs`

---

### Milestone 3：Inspector 编辑器

目标：
- 左侧 token list
- 右侧 inspector
- 支持编辑选中 token 的 rule
- UI 状态与业务状态联动

完成标准：
- 用户能选中 token
- 能切换 rule type
- 能修改 source / ratio / amount / hex
- 修改后 preview 即时更新

范围：
- `state.rs`
- `ui.rs`
- `app.rs`

---

### Milestone 4：Sample Code Preview

目标：
- 加入 code preview
- 使用 resolved semantic tokens 渲染代码
- 显示 syntax token 效果

完成标准：
- keyword / function / string / number / comment 等在 sample code 中正确着色

范围：
- `preview.rs`
- `ui.rs`

---

### Milestone 5：保存 / 加载项目

目标：
- 支持将 params + rules 保存到 TOML
- 支持重新加载项目

完成标准：
- 可以持久化当前主题工程
- 再次打开后恢复全部状态

范围：
- `persistence/`

---

### Milestone 6：导出主题文件

目标：
- 实现 exporter trait
- 实现 Alacritty exporter
- 支持导出到文件

完成标准：
- 用户可以按键导出
- 导出的文件可直接给目标工具使用

范围：
- `export/`

---

### Milestone 7：体验增强（第二阶段起点）

目标：
- presets
- reset to defaults
- 更好的 fixed color 编辑
- Expert mode 基础支持
- Auto rule 初版

---

## 22. 第一阶段明确不做的内容

为了保证 MVP 聚焦，以下内容暂不实现：

- 通用 node graph UI
- 无限嵌套表达式系统
- 完整 DSL 编辑器
- 完整 color wheel / curve editor
- 多 exporter 同时完成
- GUI 版本
- 插件系统
- 多文档工作区
- 复杂 design token inheritance 体系

---

## 23. 最终产品抽象

整个产品可以用一句话概括：

> **一个基于参数驱动 palette、基于受限规则系统生成语义 token、并可导出到具体工程配置的 ThemePrism。**

更具体地说：

- 用户编辑的不是表达式
- 用户编辑的是 token 的“生成方式”
- 系统内部用结构化 Rule Model 解析
- Preview 与 Export 统一基于 `ResolvedTheme`

---

## 24. Codex 实现优先级建议

给 Codex 的推荐执行顺序：

1. 定义所有核心 Rust 类型
2. 实现 `Color` 工具函数
3. 实现 `ThemeParams` 默认值
4. 实现 `Palette` 生成
5. 实现 `Rule` / `RuleSet`
6. 实现 evaluator
7. 做最小 TUI layout
8. 加 palette preview
9. 加 token list + inspector
10. 加 sample code preview
11. 加 persistence
12. 加 exporter

---

## 25. 实现成功标准

如果以下条件成立，就可以认为第一阶段成功：

- 可以通过 TUI 修改全局参数
- 可以编辑 token 的 rule
- 可以看到实时 preview
- 可以保存项目
- 可以导出至少一种主题文件
- 内部数据结构足够稳定，未来能扩展 GUI 与更多 exporter

---

## 26. 总结

这个系统的核心不是“画节点图”，也不是“手写 DSL”，而是：

- 参数系统
- palette 系统
- 语义 token schema
- 结构化 rule model
- rule evaluator
- preview
- exporter

其中最关键的中间层是：

```text
TokenRole -> Rule -> Evaluator -> Final Color
```

而最关键的交互层是：

```text
Token List -> Inspector Form -> Live Preview
```

这就是本工具的产品与工程核心。
