class Utils:
    def LowerInit(s: str) -> str:
        if s.Length > 1:
            s = s.Substring(0, 1).ToLower() + s.Substring(1)

        return s

    def UpperInit(s: str) -> str:
        if s.Length > 1:
            s = s.Substring(0, 1).ToUpper() + s.Substring(1)

        return s
