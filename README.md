# rain sg bot

rain sg bot will show user gifs of the current rain situation


# how this is done

Basic idea is to query api to fetch the images of the rain situation and then combine them into a gif.

Engine has one public function: generate_current_weather_condition(), this function will get the images of rain for the past 25 frames (about 2 hours previously) and save to local drive as a gif.

How the image file is saved is first create the directory where the name is based on the format of YYYYMMDD. The filename is then named based on the format of HHMM.png. The directory will separate the different files based on the date.

How the gif file is named is in the format of YYYYMMDD_YYYYMMDD.gif where the first will denote the datetime of the entire duration.

The telegram bot will then read the name of the gif and then send this gif to the bot.

folder
- src
    - engine.rs
    - image_meta.rs
    - lib.rs
    - main.rs
- img
    - 20240523  @ create a directory name based on the format of YYYYMMDD
        - 1200.png  @ create a image file name based on the format of HHMM
        - 1205.png
        - 1210.png
        - 1215.png
        - 1220.png
    - 20240524
- gif

# running the program

to run in debug mode:
```
$ TELOXIDE_TOKEN=<TELEGRAM BOT TOKEN> cargo run
```

a Dockerfile has been provided to run the program
remember to generate and add your telegram bot token before hand

```
docker build -t rain_sg:1.0.0 .
docker run -e TELOXIDE_TOKEN=<TELEGRAM BOT TOKEN> rain_sg:1.0.0
```