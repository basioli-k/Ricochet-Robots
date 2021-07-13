<?php 
function sendJSONandExit( $message )
{
    // Kao izlaz skripte pošalji $message u JSON formatu i prekini izvođenje.
    header( 'Content-type:application/json;charset=utf-8' );
    echo json_encode( $message );
    flush();
    exit( 0 );
}

$filename    = isset( $_GET['filename'] ) ? $_GET['filename'] : "";

if ($filename === "") {
    $response = [];
    $response[ 'error' ] = "Niste poslali ime filea kojeg treba dohvatiti."; 
    sendJSONandExit( $response );
}

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

$content = file_get_contents($filename);
$response[explode('.', $filename)[0]] = $content;
sendJSONandExit( $response );

?>