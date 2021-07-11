<?php
function sendJSONandExit( $message )
{
    // Kao izlaz skripte pošalji $message u JSON formatu i prekini izvođenje.
    header( 'Content-type:application/json;charset=utf-8' );
    echo json_encode( $message );
    flush();
    exit( 0 );
}

$filenames    = isset( $_GET['filenames'] ) ? $_GET['filenames'] : "";

if ($filenames === "") {
    $response = [];
    $response[ 'error' ] = "Niste poslali ime filea kojeg treba pocistiti."; 
    sendJSONandExit( $response );
}

$filenamesArray = explode(',', $filenames);

$error = "";
foreach ($filenamesArray as $filename) {
    if( !file_exists( $filename ) )
        $error = $error . "Datoteka " . $filename . " ne postoji. ";
    else
    {
        if( !is_readable( $filename ) )
            $error = $error . "Ne mogu čitati iz datoteke " . $filename . ". ";

        if( !is_writable( $filename ) )
            $error = $error . "Ne mogu pisati u datoteku " . $filename . ". ";
    }
}

if( $error !== "" )
{
    $response = [];
    $response[ 'error' ] = $error;

    sendJSONandExit( $response );
}

foreach ($filenamesArray as $filename) 
    file_put_contents($filename, "");

$response = []; 
$response[ 'response' ] = "Uspjesno pobrisano"; 
sendJSONandExit( $response );

?>