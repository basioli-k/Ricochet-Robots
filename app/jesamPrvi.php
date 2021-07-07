<?php 
// define( '__SITE_PATH', realpath( dirname( __FILE__ ) ) );
// define( '__SITE_URL', dirname( $_SERVER['PHP_SELF'] ) );

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
if (count(explode(",", $msg)) === 2) {
    $response = [];
    $response[ 'prvi' ] = true;
    sendJSONandExit( $response );
} else {
    $response = [];
    $response[ 'prvi' ] = false;
    sendJSONandExit( $response );
}

?>