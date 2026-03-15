# These should NOT be flagged

# Already using endless range
arr[1..]
arr[2..]

# Using beginless range
arr[..3]

# Different upper bound expression (not length/size - 1)
arr[1..arr.length - 2]
arr[1..10]
arr[1..n]

# Accessing a single element
arr[0]
arr[-1]
