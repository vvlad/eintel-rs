require "fileutils"
require "pathname"
require "digest/md5"
module Sde
  SDE_PATH = Pathname.new("./sde")
  module_function

    def market_groups
      @market_groups ||= cached("market_groups") do
        market_groups = YAML.load_file(SDE_PATH.join("bsd/invMarketGroups.yaml")).each_with_object({}) do |group, object|
          object[group.fetch("marketGroupID")]= group
        end
      end
    end

    def load_file(path)
      cache_key = Digest::MD5.hexdigest(path)
      cached(cache_key) { YAML.load_file(SDE_PATH.join(path)) }
    end

    def cached(key)
      cache_file = Pathname.new("tmp/cache/#{key}")
      return Marshal.load(cache_file.read) if cache_file.exist?
      puts "Cache miss for #{key}"
      object = yield
      FileUtils.mkdir_p cache_file.dirname
      cache_file.write(Marshal.dump(object))
      object
    end
end
