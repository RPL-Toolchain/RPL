# SafeDrop RPL 建模工作汇报

本次工作的目标是统一 SafeDrop 相关 RPL 模式，把具体 unsafe API 选择放到模式里。

观察 MIR inline 后，释放后使用主要可以归约为两类形式：

```text
<函数路径>::precondition_check(.., $ptr, ..)
```

因此 UAF 不再由分析器硬编码整组源码 API。源码层调用和可命名的
`precondition_check` 都直接在模式中写函数路径；内联后的 `Box::from_raw` 直接匹配
`ptr -> NonNull -> Unique -> Box` 构造链。

```text
源码调用 -> 直接匹配函数路径
内联调用 -> 直接匹配 precondition_check 路径或构造链
```

统一后的 UAF 模式可以概括为：

```text
已释放指针 -> *p 直接访问
已释放指针 -> 指定 unsafe API / precondition_check 的指定指针实参
已释放指针 -> 创建共享引用或可变引用
已释放指针 -> 传给实际使用该参数的本地函数
```

本地函数摘要递归传递参数使用和释放效果；正常返回与展开清理分别传播。

DF 的修改按 SafeDrop 的释放语义建模：

```text
MIR drop
dealloc / drop_in_place / release / destroy
```

```text
已释放指针 -> 再次释放
```
