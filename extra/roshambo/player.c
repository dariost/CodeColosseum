#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <string.h>

int main() {
    srand(time(NULL));
    const char* move[] = {"ROCK", "PAPER", "SCISSORS"};
    char* pipein = getenv("COCO_PIPEIN");
    char* pipeout = getenv("COCO_PIPEOUT");
    FILE* fin;
    FILE* fout;
    if(pipein && pipeout) {
        fin = fopen(pipein, "r");
        fout = fopen(pipeout, "w");
    } else {
        fin = stdin;
        fout = stdout;
    }
    char name[2][64];
    char opmove[64];
    int rounds;
    fscanf(fin, "%s", name[0]);
    fscanf(fin, "%s", name[1]);
    fscanf(fin, "%d", &rounds);
    for(int i = 0; i < rounds; i++) {
        fprintf(fout, "%s\n", move[rand() % 3]);
        fflush(fout);
        fscanf(fin, "%s", opmove);
        if(strcmp(opmove, "RETIRE") == 0) {
            break;
        }
    }
    return EXIT_SUCCESS;
}
