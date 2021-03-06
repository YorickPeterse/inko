# frozen_string_literal: true

module Inkoc
  module TypeSystem
    class SelfType
      include Type
      include NewInstance

      def new_instance(*)
        self.class.new
      end

      def resolve_self_type(self_type)
        self_type
      end

      def type_name
        'Self'
      end

      def self_type?
        true
      end

      def type_compatible?(other, _)
        other.self_type?
      end
    end
  end
end
