class TestAbc:
    def __init__(self, param1, param2):
        self.param1 = param1
        self.param2 = param2
        self.sum = 0

    def add(self, num1, num2):
        sum_ = num1 + num2
        self.sum += sum_
        return sum_
