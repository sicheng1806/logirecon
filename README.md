# logirecon

```mermaid
stateDiagram-v2
    Loading --> Loaded
    state "Loaded{logistics, details}" as Loaded {
        Run --> [*]: 运行对账程序
        LogisticsChanged(id) --> Logistic
        DetailChanged(id) --> Detail
    }

    state "Logistic{}" as Logistic {
        AddSheet --> [*]: 添加Sheet
    }

    state "Detail{}" as Detail {
        
    }
```
