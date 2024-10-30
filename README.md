# Lightning Warning System (⚡🌩️📡)

A Rust-based program which monitors and predicts lightning activity in scandinavia using data from MET Norway (Frost API). The results are stored in Supabase.

# Features
1. Frost API interface in Rust
2. Finding lightning near registered user locations every 10 seconds 
3. Clusters lightning storms by running a [density-based clustering non-parametric algorithm (DBSCAN)](https://en.wikipedia.org/wiki/DBSCAN) every minute
4. Calculates polygon describing a convex hull of the lightning clusters using the [Graham's scan algorithm](https://en.wikipedia.org/wiki/Graham_scan)

# Prerequisites
You will need to setup a Supabase project for this program to run properly. You can do that by setting up the required tables as defined in the structs in the `src/db.rs` file. Other than that;

* Rust (latest stable version)
* Access to [FROST API](https://frost.met.no/howto.html)

## Environment Variables
```
FROST_API_CLIENT=your_frost_client_id
FROST_API_SECRET=your_frost_api_secret
SUPABASE_URL=your_supabase_url
SUPABASE_API_SERVICE_ROLE=your_supabase_public_key
```