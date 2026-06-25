# logirecon

## v0.2.0

- [x] 重构模块为更小的crate: 
  - [x] logirecon_core: 核心接口层, 无此需求
  - [x] logirecon: 实现层
    - [x] 提取公共 polars 表达式，无此需求
    - [x] 提取 run_reconsiliation 纯函数，在 让 get_reconcile 函数中的 IO 和 计算工程边界清晰
  - [x] logirecon_iced: 基于 iced 实现的用户界面
  - [ ] logirecon_dioxus: 基于 dioxus 实现的用户界面

## v0.0.1

- [x] Excel 文件读取
- [x] 万邦账单输入
- [x] 我方头程明细输入
- [x] 运费和报关费表格生成
- [x] 对账功能实现
- [x] 其他账单类型补全
- [x] 逻辑错误检查
- [x] 用户界面设计
