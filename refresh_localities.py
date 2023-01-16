# Use when the database has already been initialised
import sqlite3
import stance_grouping
import localities

# Connect to stops database
db = sqlite3.connect("bus-site/stops.sqlite")

localities.init_db(db)

# Regenerate localities.json (TODO create an actual method)
stance_grouping.main()

# Recreate stops
localities.insert_stops(db)

# Remove unused stops
print("Removing unused stops")
db.execute("SELECT DISTINCT stances.stop FROM stop_times INNER JOIN stances ON stances.code=stop_id UNION SELECT stances.stop FROM stances WHERE crs IS NOT NULL;")

# Rebuild stops_search table
print("Rebuilding stops_search table")
db.execute("DROP TABLE IF EXISTS stops_search;")
db.execute("CREATE VIRTUAL TABLE stops_search USING fts5(name, parent, qualifier, id UNINDEXED);")
db.execute("INSERT INTO stops_search(name, parent, qualifier, id) SELECT stops.name, stops.locality_name, qualifier, stops.id FROM stops INNER JOIN localities l on l.code = stops.locality;")

# Write to database
db.commit()
db.close()
