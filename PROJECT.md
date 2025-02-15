This project in a CLI tool meant to help do one shot generations. How it works is by scanning a directory of files, and using RAG to keep a small sql lite database of files to vectors. It watches for file changes and updates that RAG database over time ( not re-RAGifying a document unless there is an actual change.)

`luckyshot --watch`

the other functionality of luckshot is to run aider with a task.  It does this by looking at the request, and then searching up the relevant files according to RAG it thinks will be  the ones to modify.  Then it will run aider just once.

`luckshot make the background green`