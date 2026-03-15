# These should NOT be flagged

# Pure uppercase range
str.match?(/[A-Z]/)

# Pure lowercase range
str.match?(/[a-z]/)

# Digit range
str.match?(/[0-9]/)

# Combined but not mixed-case ranges
str.match?(/[A-Za-z]/)
str.match?(/[a-zA-Z0-9]/)

# Single character class
str.match?(/[abc]/)
