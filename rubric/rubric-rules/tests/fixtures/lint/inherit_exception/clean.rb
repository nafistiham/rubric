class FooError < StandardError
end

class BarError < RuntimeError
end

class BazError < MyApp::Error
end

# class BadError < Exception is fine in comments
