# Multi-line ternary operators that should be flagged

result = some_condition ?
  value_if_true :
  value_if_false

x = a > b ?
  a :
  b

foo = bar.nil? ? nil : bar  # single-line, NOT flagged
