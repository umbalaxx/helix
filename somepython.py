class Calculator:
    def __init__(self, param1, param2):
        self.param1 = param1
        self.param2 = param2
        self.sum = 0

    def add(self, num1, num2):
        sum_ = num1 + num2
        self.sum += sum_
        return sum_

    def subtract(self, num1, num2):
        return num1 - num2

    def divide(self, num1, num2):
        if num2 == 0:
            raise ValueError("Cannot divide by zero")
        return num1 / num2

    def modulo(self, num1, num2):
        return num1 % num2

    def multiply(self, num1, num2):
        return num1 * num2

    def reset(self):
        self.sum = 0

    def power(self, num1, num2):
        return num1**num2

    def square_root(self, num1):
        return num1**0.5

    def factorial(self, num1):
        if num1 == 0:
            return 1
        else:
            return num1 * self.factorial(num1 - 1)

    # Some other functions

    def is_prime(self, num1):
        if num1 <= 1:
            return False
        for i in range(2, int(num1**0.5) + 1):
            if num1 % i == 0:
                return False
        return True

    def is_even(self, num1):
        return num1 % 2 == 0

    def is_odd(self, num1):
        return num1 % 2 != 0
