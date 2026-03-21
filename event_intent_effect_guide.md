# Event / Intent / Effect / Update 概念速记

## 1. 这份文档讲什么

这份文档专门回答下面这个问题：

> 为什么 UI 架构里经常会出现 `Event -> Intent -> Update -> Effect` 这条链？  
> 这些概念分别是什么？边界在哪里？

它是上一份跨平台 UI 架构文档的补充版，但更短，更聚焦。

---

## 2. 先给最短结论

你可以把这几个词理解成不同层级的“发生了什么”。

### `Native Event`

平台原始输入。

例如：

- TUI: `KeyCode::Left`
- GUI: 鼠标点击按钮
- GUI: slider 被拖到 `0.42`

### `Control Event`

某个控件层面的语义事件。

例如：

- “这个 slider 的值变成了 `0.42`”
- “这个 picker 选中了 `accent_3`”
- “这个输入框提交了 `#C586C0`”

### `Intent`

用户在产品语义层面想做什么。

例如：

- `SetParam(SelectionMix, 0.42)`
- `SetSource(Background, Accent3)`
- `CommitFixedColor(Keyword, "#C586C0")`
- `TriggerSave`

### `Update`

收到 intent 之后，如何修改状态。

例如：

- 改 `params.selection_mix`
- 改 `rules.Background`
- 打开 picker
- 关闭输入框

### `Effect`

状态更新之后，需要外部世界执行的动作。

例如：

- 写文件
- 弹出系统文件选择器
- 复制到剪贴板
- 发网络请求

---

## 3. 为什么这些层要分开

因为它们并不是一回事。

## 3.1 Native Event 不等于用户意图

例如：

- TUI 里按 `LeftArrow`
- GUI 里拖 slider
- GUI 里滚轮滚一下

这些原始事件看起来完全不同。

但它们最后可能表达的是同一件事：

```text
AdjustScalar(SelectionMix, -1)
```

或者：

```text
SetScalar(SelectionMix, 0.42)
```

所以原始事件只是“输入来源”，不是“产品语义”。

---

## 3.2 Effect 也不应该混在 Update 里

例如：

用户按了保存。

你可以有两种写法。

### 写法 A：直接在事件处理函数里写文件

```text
if key == CtrlS {
  save_to_disk();
}
```

### 写法 B：先产生意图和 effect

```text
Intent::TriggerSave
```

然后：

```text
update(state, Intent::TriggerSave) -> [Effect::SaveProject(path)]
```

后者更好，因为：

- 状态逻辑和外部副作用分开了
- 更好测试
- 更好跨平台
- 更好替换实现

---

## 4. 一个更完整的层次图

可以把它想成这样：

```text
Native Event
  -> Event Adapter
  -> Control Event（可选）
  -> Intent
  -> Update(State)
  -> Effects
  -> Effect Runner
```

其中：

- `Native Event` 一定是平台相关
- `Intent` 一定应该尽量平台无关
- `Effect` 也是平台无关的描述
- 真正执行 effect 的 runner 是平台相关

---

## 5. `Control Event` 是不是必须有

不一定。

这是个中间层，有时很有用，有时可以省略。

## 5.1 什么是 `Control Event`

它比 `Native Event` 高一级，但还没到“产品意图”。

例如：

- `SliderChanged(control_id, 0.42)`
- `PickerOptionChosen(control_id, "accent_3")`
- `TextCommitted(control_id, "#C586C0")`

这层是“控件语义”，还不是“领域语义”。

## 5.2 什么时候值得有

当你有：

- 很多控件类型
- 一套通用表单系统
- GUI/TUI/Web 多端渲染
- 通用控件库

这时候把 `Native Event -> Control Event -> Intent` 分开会更清楚。

## 5.3 什么时候可以省

如果项目还不大，你可以直接：

```text
Native Event -> Intent
```

比如：

- TUI 的 Enter 在某个 source 字段上
  -> `OpenSourcePicker(Background)`

中间不一定非要再包一层 `ControlEvent`。

### 对你当前项目的建议

你这个项目现在其实可以先走：

```text
Native Event -> Intent -> Update -> Effect
```

等控件系统更成熟，再决定是否拆出 `ControlEvent`。

---

## 6. 这几个概念，在你的主题生成器里分别长什么样

下面用你现在的程序举例。

## 6.1 Native Event

### TUI 版

- `LeftArrow`
- `RightArrow`
- `Enter`
- `Esc`
- `Char('e')`
- `Char('s')`

### 未来 GUI 版

- slider drag
- button click
- picker selection
- text field commit

---

## 6.2 Intent

建议长成这种“产品语义动作”：

```text
enum Intent {
  MoveFocusNext,
  MoveFocusPrev,
  SelectNextToken,
  SelectPrevToken,
  SelectNextParam,
  SelectPrevParam,
  SelectNextInspectorField,
  SelectPrevInspectorField,

  AdjustParam(ParamKey, Delta),
  SetParam(ParamKey, f32),

  SetRuleKind(TokenRole, RuleKind),
  AdjustMixRatio(TokenRole, Delta),
  SetMixRatio(TokenRole, f32),
  SetAdjustAmount(TokenRole, f32),
  SetAdjustOp(TokenRole, AdjustOp),
  SetFixedColor(TokenRole, Color),
  SetRuleSource(TokenRole, SourceSlot, SourceRef),

  OpenTextInput(TextInputTarget),
  UpdateTextInput(StringEdit),
  CommitTextInput,
  CancelTextInput,

  OpenSourcePicker(SourcePickerTarget),
  FilterSourcePicker(StringEdit),
  MoveSourcePickerSelection(Delta),
  ApplySourcePickerSelection,
  CloseSourcePicker,

  SaveProject,
  LoadProject,
  ExportTheme,
  ResetProject,
}
```

这层不该知道：

- 是哪个键触发的
- 是点按钮还是拖 slider

只关心用户在产品层到底想做什么。

---

## 6.3 Update

`Update` 就是：

```text
fn update(state: &mut AppState, intent: Intent) -> Vec<Effect>
```

它做的事通常包括：

- 改共享状态
- 触发重算
- 打开 / 关闭 modal
- 设置错误消息
- 产出 effect

例如：

- `OpenSourcePicker` -> 修改 `ui.source_picker = Some(...)`
- `SetParam` -> 修改 `domain.params` 并重新计算 theme
- `SaveProject` -> 不直接写文件，而是返回 `Effect::SaveProject(...)`

---

## 6.4 Effect

Effect 是“要让外部世界去做”的事情。

例如：

```text
enum Effect {
  SaveProject(PathBuf),
  LoadProject(PathBuf),
  ExportTheme {
    exporter: ExportTarget,
    path: PathBuf,
  },
  ShowError(String),
  OpenFileDialog(FileDialogPurpose),
}
```

注意：

Effect 不是“已经做完了”。

它只是：

> 共享层要求外部系统去做这件事。

真正执行的是 effect runner。

---

## 7. 为什么即使只有 GUI，这一层也常常值得有

你前面问了一个关键点：

> 如果全都是 GUI，因为 GUI 的 event 看起来比较一致，是不是就不需要 `Event -> Intent` 了？

答案是：

> **不是因为 TUI 才需要这层。即使全 GUI，这层也常常是对的。**

原因如下。

## 7.1 GUI 事件其实也不真的“一致”

GUI 里你以为是统一的，但实际上：

- button click
- slider changed
- text changed
- text committed
- menu item triggered
- toolbar action
- shortcut
- drag finished

这些本身也很杂。

如果直接在每个 callback 里写业务逻辑，状态会慢慢分散到各处。

---

## 7.2 同一个意图经常有多种入口

例如保存：

- 按钮点 Save
- 菜单点 Save
- `Cmd+S`
- 以后自动保存

它们都应该通往同一个逻辑：

```text
Intent::SaveProject
```

不该每个入口自己写一遍保存逻辑。

---

## 7.3 控件 callback 很容易把业务污染掉

如果你在 GUI callback 里直接：

- 校验
- 改 domain state
- 写文件
- 更新多个局部 UI 状态

那么平台代码会越来越重。

而你最关心的恰恰是：

> 平台相关代码尽可能少

所以恰恰更应该把 callback 降级成：

```text
native callback -> intent
```

---

## 8. 什么时候可以不显式搞一层 Intent

也不是说所有项目都必须严密分层。

如果满足下面条件：

- 界面很小
- 状态很浅
- 没有多输入方式
- 没有复杂 modal / picker / 校验
- 不打算跨平台

那你完全可以简单一点：

```text
callback -> mutate local state
```

这在小工具里没问题。

但如果项目像你的主题生成器这样：

- 状态明显在增长
- 有多种编辑模式
- 有 picker / input mode / save/load/export
- 以后还要 GUI

那 `Intent / Update / Effect` 就会越来越值。

---

## 9. 一个最小可用版本，不需要搞太重

你现在不用一上来就做成超完整 Redux。

对这个项目，我建议最小版本就用这 4 个东西：

## 9.1 `Intent`

平台无关用户意图。

## 9.2 `AppState`

共享状态。

## 9.3 `update(state, intent) -> effects`

共享状态机。

## 9.4 `Effect`

对外部副作用的描述。

就够了。

`ControlEvent` 可以暂时不单独拆。

---

## 10. 一个你可以直接记住的边界判断法

当你拿到一段逻辑，不知道该放哪层时，可以问自己：

## 10.1 这是“平台原始输入”吗？

如果是：

- 按键
- 鼠标
- toolkit callback

那它属于平台层。

## 10.2 这是“用户想做什么”吗？

如果是：

- 保存
- 调大对比度
- 打开 source picker
- 提交 hex color

那它属于 intent 层。

## 10.3 这是“状态怎么变化”吗？

如果是：

- focus 切换
- buffer 改变
- rule 被替换
- picker 打开/关闭

那它属于 update 层。

## 10.4 这是“要外部世界干活”吗？

如果是：

- 写文件
- 打开 dialog
- 调系统 API

那它属于 effect / effect runner。

---

## 11. 一句话总结

可以把整件事记成这句话：

> **Event 是平台怎么告诉你“发生了什么”；Intent 是用户在产品里想做什么；Update 是系统内部怎么改状态；Effect 是需要外部世界替你做什么。**

再压缩一点：

```text
Event = 输入来源
Intent = 产品语义
Update = 状态变化
Effect = 外部副作用
```

如果你能把这四件事分开，平台相关代码就会自然缩小很多。
