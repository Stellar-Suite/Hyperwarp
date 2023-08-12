int rust_launch() {
    printf("You are meant to call this with Hyperwarp loaded into the process.\n");
    return 0;
}

int main (int argc, char *argv []) {
    // we can get the args in rust through a std module.  
    return rust_launch();    
}