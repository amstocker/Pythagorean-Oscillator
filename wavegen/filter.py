class LPF_1P:
    def __init__(self, param):
        self.param = param
        self.acc = 0
        self.acc_delayed = 0

    def process(self, frames):
        for frame in frames:
            self.acc = (1 - self.param) * frame + self.param * self.acc_delayed
            self.acc_delayed = self.acc
            yield self.acc