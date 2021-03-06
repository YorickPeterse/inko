# frozen_string_literal: true

module Inkoc
  class SymbolTable
    include Inspect
    include Enumerable

    attr_reader :symbols, :mapping
    attr_accessor :parent

    def initialize(parent = nil)
      @symbols = []
      @mapping = {}
      @parent = parent
      @unique_names = false
      @remapped_names = {}
    end

    def with_unique_names
      @unique_names = true
      return_value = yield
      @unique_names = false
      @remapped_names = {}

      return_value
    end

    def add_symbol(symbol)
      @symbols << symbol
      @mapping[symbol.name] = symbol
    end

    def define(name, type, mutable = false)
      if @unique_names
        symbol_name = name + object_id.to_s
      else
        symbol_name = name
      end

      symbol = Symbol.new(symbol_name, type, @symbols.length, mutable)

      @symbols << symbol
      @mapping[symbol_name] = symbol

      if @unique_names
        @remapped_names[name] = symbol
      end

      symbol
    end

    def reassign(name, type)
      @symbols[name].type = type
    end

    def names
      @mapping.keys
    end

    def each
      @symbols.each do |value|
        yield value
      end
    end

    def [](name_or_index)
      symbol =
        if name_or_index.is_a?(Integer)
          @symbols[name_or_index]
        else
          @mapping[name_or_index] || @remapped_names[name_or_index]
        end

      symbol || NullSymbol.singleton
    end

    def slice(range)
      @symbols[range] || []
    end

    def lookup_with_parent(name_or_index)
      source = self
      depth = -1

      while source
        found = source[name_or_index]

        return [depth, found] if found.any?

        depth += 1
        source = source.parent
      end

      [-1, NullSymbol.singleton]
    end

    def lookup_in_root(name_or_index)
      source = self
      depth = -1

      while source.parent
        depth += 1
        source = source.parent
      end

      [depth, source[name_or_index]]
    end

    def defined?(name)
      lookup_with_parent(name)[1].any?
    end

    def last
      @symbols.last
    end

    def any?(&block)
      @symbols.any?(&block)
    end

    def empty?
      @symbols.empty?
    end

    def length
      @symbols.length
    end

    def ==(other)
      other.is_a?(self.class) &&
        mapping == other.mapping &&
        parent == other.parent
    end
  end
end
