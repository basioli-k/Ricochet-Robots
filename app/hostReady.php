<?php
function sendJSONandExit( $message )
{
    // Kao izlaz skripte pošalji $message u JSON formatu i prekini izvođenje.
    header( 'Content-type:application/json;charset=utf-8' );
    echo json_encode( $message );
    flush();
    exit( 0 );
}


$filename  = 'ready.log';

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


if( $error !== "" )
{
    $response = [];
    $response[ 'error' ] = $error;

    sendJSONandExit( $response );
}

$msg = file_get_contents($filename);

while($msg != "1")
{
    usleep( 10000 ); // odspavaj 10ms da CPU malo odmori :)
    clearstatcache();
    $msg = file_get_contents($filename);
}

$response = array();

sendJSONandExit( $response );

?>