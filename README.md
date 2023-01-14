Bus Boards
==

<details>
  <summary>Screenshots</summary>
  <p>
  
  ![Stop](https://user-images.githubusercontent.com/15062976/211429115-69a7fee1-df25-4ac2-98d7-85025b812981.png)
  
  ![Service](https://user-images.githubusercontent.com/15062976/211429125-288e1ef8-c1be-4444-98c8-0b42178bbc61.png)
  </p>
</details>

## What is it?

Bus Boards is an experiment in showing bus departures akin to [train 
departures](https://www.realtimetrains.co.uk/search/simple/gb-nr:YRK), to see
if it makes them any easier for the average traveller. Bus stop data is 
stored per-stance, so it does this by grouping stances together into stops 
to show a departure board for combined locations.


## Why?
Buses are difficult to figure out. When searching Traveline for live times at 
Edinburgh Bus Station, I'm presented with 18 different stances. An irregular 
bus user may not memorise a bus's route number, nevermind the stance. Google 
Maps and Apple Maps do a decent job of this nowadays, but there is scope for 
more advanced features.

Furthermore, buses are often part of someone's wider journey. If you're 
getting off the train at Leuchars, your next journey may very well be a bus 
to St Andrews, and you may just want to know when that bus is going to arrive. 
Whilst National Rail shows "buses", they exist more for 
ticketing and routing purposes than actual journey help, and have no 
connection to the actual timetable or current journey data.

Journey planners do exist - they're everywhere now, from the Trainline app to 
most ticket machines in stations. However, sometimes it's just easier to 
view listings in a board, without having to search for and find your journey,
which may or may not be a suggested route.

And don't get me wrong - [bustimes.org](https://bustimes.org) is a *fantastic* 
site. It collates data from lots of sources, presents timetables in a clear 
way and has a great live map. It's more accurate than this ever will be. 
This just does things in a different way, that's all.


## Current features
- Search for stops
- View all scheduled departures for a 2-hour period
  - View services in the past or future that exist in the current dataset
  - Filter by stance and/or operator
- See the realtime location of buses where available


## Flaws

### Stance grouping isn't perfect
The NaPTAN database isn't amazing. For example, bus stations stances are named
inconsistently, whether it comes to capitalisation, or even the usage of the
term "Bus Station". That's not even between different bus stations or local
authorities either - that happens within the same location! As a result, there's
a lot of patterns to look out for, all of which conflict with each other, 
making the act of grouping stops no easy feat. If it was, it would've been 
done already. As a result, some stops exist under multiple names.

### Some stops may not belong together
The concept of grouping may not even be ideal for some stops. [Take a look at 
this map of stops within Leeds City Centre by First Bus](https://www.firstbus.co.uk/sites/default/files/public/maps/1.%20Leeds%20City%20Centre%20Map.pdf).
Civic H is 600m away from Civic N - hardly an easy interchange! However, 
keeping them under the same name is still the clearest way for them to exist.

### Using converted GTFS
By law, bus timetable data feeds are published in the TransXChange format 
created by the UK government. This is a very detailed XML format which isn't the
easiest to use, so the data is also offered in the simpler GTFS format created
by Google. Presumably due to the complexity of TransXChange (or possibly due 
to overlap between first-party BODS data and the Traveline National Dataset), 
the datasets don't convert perfectly over to GTFS, so there are some
inaccuracies in the timetable:
- Bus duplication - at Leeds Bus Station, most Flyer services are also 
  registered as Coastliner services.
- Wrong start/end times - the East Yorkshire Scarborough-Helmsley route 
  shows services on the X28, which no longer exists in the new timetable.

bustimes.org uses TransXChange files directly, so contains a more accurate 
view of services. For the most accurate data, this project would likely have 
to do the same.

### There are a lot of buses
Why don't buses do things like trains? The most basic reason... there are 
more buses than trains! The [Stagecoach 143 from Manchester - West Didsbury](https://bustimes.org/services/143-manchester-west-didsbury-2) 
alone will operate up to every 5 minutes during the day on a bus corridor so 
busy it has [its own Wikipedia page](https://en.wikipedia.org/wiki/Wilmslow_Road_bus_corridor).
This can often lead to a very crowded departures list. The stance and 
operator filters help in part to address this.


## Setup

The repository is set up with a root directory and bus-site directory to 
accommodate for a Python module with a JavaScript submodule in IntelliJ. 
There are better/clearer ways to organise this, so this may change.
Setup is currently quite lengthy and may be more automated soon.

1. Download the archive of [all bus timetables data in GTFS format](https://data.bus-data.dft.gov.uk/timetable/download/gtfs-file/all/)
   from the Bus Open Data Service and place it in bus-site/gtfs. You may 
   need a free account to access this.
2. Place the [NaPTAN in CSV format](https://naptan.api.dft.gov.uk/v1/access-nodes?dataFormat=csv)
   into the root directory as `Stops.csv`, and the [NPTG in XML format](https://naptan.api.dft.gov.uk/v1/nptg)
   into the root directory as `NPTG.xml`.
3. Install Python dependencies in the root directory using `pip3 install -r 
   requirements.txt`. If there are conflicts, look into using a `virtualenv`.
4. Install JavaScript dependencies in `bus-site` using `npm install` in the 
   `bus-site` directory.
5. Generate stop data by running `stance_grouping.py`.
6. Insert stop data into the database by running `localities.py`.
7. In the `bus-site` directory, run `npm run gtfs`, which will run 
  `import_gtfs.ts`.
8. [Register for OpenLDBWS](http://realtime.nationalrail.co.uk/OpenLDBWSRegistration/)
   and store the API token in an environment variable named `DARWIN_API_KEY`.
9. Run the site from the `bus-site` directory using `npm run dev` or `npm 
   run preview`.


## Next steps

- [ ] Fix unmerged stances (e.g. for Seacroft and Wakefield Bus Stations)
- [ ] Manual route fixes (e.g. remove Flyer/Coastliner duplicates, change 888 
  destination from "Vauxhall" to "Luton Airport Parkway")
- [ ] Delay prediction
  - [ ] Show on departures board 
- [ ] Use route shape data for improved locating
- [ ] Use data from other sources where possible (e.g. TfWM, TfGM)
- [ ] Explore the use of using TransXChange and the Traveline National 
  Dataset directly (and not GTFS conversions) for improved accuracy 
- [x] Use National Rail Darwin API for integrated bus/train departures
  - [ ] Expand integration to include trams etc.
  - [ ] Improve location finding by finding stops in the same locality 
    within very close range
- [ ] View arrivals
- [ ] Locality pages (view all stops in a locality)
- [x] Host online
- [ ] Filter boards by destination or calling points
- [ ] Show maps of all the stances grouped within a stop
- [ ] Simplify and automate more of setup
  - [ ] Explore downloading individual region files and inserting rows into the 
    database in parallel to speed up initialisation
  - [ ] Write database migrations


## Data sources
- Bus Open Data Service (GTFS timetables include Traveline National Dataset 
  and TfL data) (https://data.bus-data.dft.gov.uk/search/)
- NaPTAN and NPTG (https://beta-naptan.dft.gov.uk/)

All data sources are made available under the [Open Government License v3.0](https://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/).
