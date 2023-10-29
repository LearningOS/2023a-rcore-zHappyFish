1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

### 编程作业

    spawn 的实现：首先调用 get_app_data_by_name 获取程序信息，再调用TaskControlBlock::new 创建一个新任务，之后设置好父子进程的关系。最后将子进程加入TASK_MANAGER的ready_queue即可

    stride 调度算法: 首先在 TaskControlBlockInner 中添加两个成员 stride 和 priority。之后修改 task/manager.rs 中的 fetch 方法，取出stride最小的任务，并将stride加上pass（BigStride / priority）

    值得一提的是，执行exec后需要将任务的stride和priority重置。

### 问答作业

stride 算法深入

stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。

实际情况是轮到 p1 执行吗？为什么？

    不是，由于超出了8bit 无符号整形的上限，p2执行后它的stride将溢出，变成4，所以实际上仍是p2执行

我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， 在不考虑溢出的情况下 , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。

为什么？尝试简单说明（不要求严格证明）。

    当优先级 >= 2时，pass最大为BigStride / 2
    假设 x1 <= x2 <= x3 .... <= xn， 且 xn - x1 <= BigStride / 2
    那么我们会选出x1,并加上pass，此时有两种情况，加上pass后x1成为最大值，那么必然有STRIDE_MAX – STRIDE_MIN <= BigStride / 2，因为 pass <= BigStride / 2。第二种情况x1加上pass后不是最大值，假设此时的最小值为 x， 那么 xn - x <= xn - x1 <= BigStride / 2,同样满足STRIDE_MAX – STRIDE_MIN <= BigStride / 2
    由此递推可知只要一开始满足STRIDE_MAX – STRIDE_MIN <= BigStride / 2，那么之后就一定满足STRIDE_MAX – STRIDE_MIN <= BigStride / 2，而初始条件下stride都相同，显然满足STRIDE_MAX – STRIDE_MIN <= BigStride / 2

已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。

```rs

use core::cmp::Ordering;

const BIGSTRIDE:u64 = 255;
struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 < other.0 && (other.0 - self.0) <= BIGSTRIDE / 2 {
            Some(Ordering::Less)
        }else if self.0 > other.0 && (self.0 - other.0) <= BIGSTRIDE / 2 {
            Some(Ordering::Greater)
        }else {
            None
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

#[cfg(test)]
mod test {
    use super::Stride;
    #[test]
    fn cmp() {
        let x1 = Stride(125);
        let x2 = Stride(129);
        let x3 = Stride(255);
        assert_eq!(x1 < x2, true);
        assert_eq!(x2 < x3, true);
        assert_eq!(x1 < x3, false);
    }
}
```

        

    
