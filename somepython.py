class Calculator:
    def __init__(self, param1, param2):
        self.param1 = param1
        self.param2 = param2
        self.sum = 0

    def add(self, num1, num2):
        sum_ = num1 + num2
        self.sum += sum_
        return sum_

    def calculate_sum(self, num1, num2):
        return num1 + num2

    def divide(self, num1, num2):
        return num1 / num2

    def multiply(self, num1, num2):
        return num1 * num2
