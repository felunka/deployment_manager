class CreateNodes < ActiveRecord::Migration[8.1]
  def change
    create_table :nodes do |t|
      t.string :hostname, null: false
      t.string :ip, null: false
      t.string :api_url, null: false
      t.integer :port, null: false, default: 443
      t.integer :node_status, default: 0, null: false
      t.string :key, null: false

      t.timestamps
    end
  end
end
