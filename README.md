# Inventory Management Software (working on a better title)

Very early work in progress.  Utilizes rusqlite to manage a database.  There is a ratatui for interacting with it.  Currently very minimal functionality.  Currently, the databases are all done in memory to provide a consistent, repeatable enviornemt for development.  This means that the data is not stored anywhere.  Before actual use, this needs to be swapped to use the disk.


## Application Structure
There is an app struct in main.rs.  This is responsible for managing the main loop of the program.  The database and related structures are housed in db.rs.  Different App pages and functionalities are implements as Applets.  Upon running the program, it loads a top menu.  This allows you to select a new applet to run.  The App stores applets in a Vec(LIFO) stack, allowing the program to return to a previous applet while keeping the existing context.  DB calls are minimized by only refreshing when the top applet changes, as that is the only time DB data could have changed.
