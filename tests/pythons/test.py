class Test1:
    @classmethod
    def prt(cls):
        print(cls.__name__)


class Test(Test1):
    def __init__(self):
        self.a = 1

Test.prt()