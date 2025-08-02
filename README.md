# squid-orbit-simulator
A hobby orbit simulator, especially for LEO CubeSat mission ops planning

## Goals

1. Predict satellite state (velocity vector and position), propogating a TLE, using the [SGP4 model](https://en.wikipedia.org/wiki/SGP4).
2. Evaluate link windows.
3. Evaluate power generation/sunlight conditions.
4. Export granular data for further processing.
5. Simulate how propulsion can be used to modify an orbit, especially with the goal of forcefully changing the orbit into a tight eliptical orbit (100km perigee, large apogee) and then back to a more "normal" orbit.

## Data Notes

* Search for NORAD ID from satellite name: https://celestrak.org/satcat/search.php
* Easiest lookup for historical TLEs: https://satdb.ethz.ch/
* Sanity check decoder for TLEs: https://sat-tle-parser.netlify.app
