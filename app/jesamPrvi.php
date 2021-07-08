<?php 

function sendJSONandExit( $message )
{
    // Kao izlaz skripte pošalji $message u JSON formatu i prekini izvođenje.
    header( 'Content-type:application/json;charset=utf-8' );
    echo json_encode( $message );
    flush();
    exit( 0 );
}

$filename  = "../controller/usernames.log";

$error = "";
if( !file_exists( $filename ) )
    $error = $error . "Datoteka " . $filename . " ne postoji. ";
else
{
    if( !is_readable( $filename ) )
        $error = $error . "Ne mogu čitati iz datoteke " . $filename . ". ";

    if( !is_writable( $filename ) )
        $error = $error . "Ne mogu pisati u datoteku " . $filename . ". ";
} 

// echo "prije errora " . $error;
if( $error !== "" )
{
    $response = [];
    $response[ 'error' ] = $error;

    sendJSONandExit( $response );
}

$msg = file_get_contents($filename);
// echo "tu sam" . $msg;
if (count(explode("\n", $msg)) === 2) {
    $response = [];
    $response[ 'prvi' ] = true;
    $response['tu'] = $msg;
    $response['count'] = count(explode("\n", $msg));
    sendJSONandExit( $response );
} else {
    $response = [];
    $response[ 'prvi' ] = false;
    $response['tu'] = $msg;
    $response['count'] = count(explode("\n", $msg));
    sendJSONandExit( $response );
}

?>