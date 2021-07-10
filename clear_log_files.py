from os import listdir
from os.path import isdir

def delete_log_content(path):
    folders = []
    for file in listdir(path):
        if isdir(file) and not file ==".git":
            folders.append(file)
            # print(file, isfile(file))
        elif not file ==".git" and ".log" in file:
            print(file)
            #prvo si ispisite da vidite da necete pobrisati nesto viska slucajno i onda odkomentirajte
            # open(path + "/" + file, 'w').close()

    for folder in folders:
        delete_log_content(path + "/" + folder)

if __name__ == "__main__":
    path = "."
    delete_log_content(path)