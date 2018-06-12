require 'fileutils'
require 'pathname'
require 'digest/md5'
module Sde
  SDE_PATH = Pathname.new('./sde')

  module_function

  def market_groups
    @market_groups ||= cached('market_groups') do
      YAML.load_file(SDE_PATH.join('bsd/invMarketGroups.yaml')).each_with_object({}) do |group, object|
        object[group.fetch('marketGroupID')] = group
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

  def ships
    ships = cached('ships-names') do
      load_file('fsd/typeIDs.yaml').select do |_id, item|
        ships_group_ids.include?(item['marketGroupID'])
      end.transform_values do |ship|
        ship.dig('name', 'en')
      end.values
    end

    ships = File.read('./data/ships.txt').lines.map(&:strip) + ships.map(&:upcase)
    ships.uniq
  end

  def ships_group_ids
    @ships_group_ids ||= begin
      ships_group_ids = []
      market_groups.each do |_id, group|
        stack = []
        loop do
          stack << group['marketGroupID']
          group = market_groups[group['parentGroupID']]
          break unless group
        end
        ships_group_ids.concat(stack) if stack.last == 4
      end

      ships_group_ids.uniq
    end
  end
end
