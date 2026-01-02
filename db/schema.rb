# This file is auto-generated from the current state of the database. Instead
# of editing this file, please use the migrations feature of Active Record to
# incrementally modify your database, and then regenerate this schema definition.
#
# This file is the source Rails uses to define your schema when running `bin/rails
# db:schema:load`. When creating a new database, `bin/rails db:schema:load` tends to
# be faster and is potentially less error prone than running all of your
# migrations from scratch. Old migrations may fail to apply correctly if those
# migrations use external dependencies or application code.
#
# It's strongly recommended that you check this file into your version control system.

ActiveRecord::Schema[8.1].define(version: 2026_01_02_201235) do
  # These are extensions that must be enabled in order to support this database
  enable_extension "pg_catalog.plpgsql"

  create_table "node_deployments", force: :cascade do |t|
    t.string "compose"
    t.datetime "created_at", null: false
    t.integer "deployment_status", default: 0, null: false
    t.integer "deployment_type", default: 0, null: false
    t.string "git_url"
    t.string "name", null: false
    t.bigint "node_id", null: false
    t.string "path", default: "/home/node_agent/<NAME>", null: false
    t.datetime "updated_at", null: false
    t.index ["node_id"], name: "index_node_deployments_on_node_id"
  end

  create_table "nodes", force: :cascade do |t|
    t.string "api_url", null: false
    t.datetime "created_at", null: false
    t.string "hostname", null: false
    t.string "ip", null: false
    t.string "key", null: false
    t.integer "node_status", default: 0, null: false
    t.integer "port", default: 443, null: false
    t.datetime "updated_at", null: false
  end

  create_table "users", force: :cascade do |t|
    t.datetime "created_at", null: false
    t.string "email", null: false
    t.string "github_pat"
    t.string "name", null: false
    t.string "password_digest", null: false
    t.datetime "updated_at", null: false
  end

  add_foreign_key "node_deployments", "nodes"
end
