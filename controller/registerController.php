<?php
class RegisterController{
    
    function index(){
        require_once __SITE_PATH . "/view/register.php";
        exit();
    }

    function newUser(){
        if( !isset( $_POST['username'] ) || !isset( $_POST['password'] ) || !isset( $_POST['email']) )
        {
            require_once __SITE_PATH . "/view/register.php";
            $this->index( 'Please input username, password and email.' );
            exit();
        }
        else if( !preg_match( '/^[A-Za-z]{3,15}$/', $_POST['username'] ) )
        {
            $warning = "The username has to have between 3 and 15 letters.";
            require_once __SITE_PATH . "/view/register.php";
            exit();
        }
        else if( !filter_var( $_POST['email'], FILTER_VALIDATE_EMAIL) )
        {
            $warning = "Invalid email adress.";
            require_once __SITE_PATH . "/view/register.php";
            exit();
        }
        else if ($_POST["password"] === ""){
            $warning = "Invalid password.";
            require_once __SITE_PATH . "/view/register.php";
            exit();
        }

        //check if user exists
        $user = Player::getPlayer( $_POST["username"], $_POST["email"] );

        if($user){
            $warning = "A user with username ". $user->username . " or email " . $user->email . " already exists.";
            require_once __SITE_PATH . "/view/register.php";
            exit();
        }

        $registration_sequence = '';
		for( $i = 0; $i < 20; ++$i )
			$registration_sequence .= chr( rand(0, 25) + ord( 'a' ) );

        Player::insert( array("username" => $_POST["username"], "password_hash" => password_hash( $_POST['password'], PASSWORD_DEFAULT ),
                    "email" => $_POST["email"], "has_registered" => 0, "registration_sequence" => $registration_sequence, "games_played" => 0,
                    "tokens_won" => 0, "games_won" => 0 ));


        $to       = $_POST['email'];
        $subject  = 'Registration confirmation';
        $message  = 'Greetings ' . $_POST['username'] . "!\nTo finish registration click the following link: ";
        $message .= 'http://' . $_SERVER['SERVER_NAME'] . htmlentities( dirname( $_SERVER['PHP_SELF'] ) ) . '/index.php?rt=register/confirmation&sequence=' . $registration_sequence . "\n";
        $headers  = 'From: rp2@studenti.math.hr' . "\r\n" .
                    'Reply-To: rp2@studenti.math.hr' . "\r\n" .
                    'X-Mailer: PHP/' . phpversion();

        $isOK = mail($to, $subject, $message, $headers);

        if( !$isOK )
            exit( 'Error while sending e-mail.' );
        
        $warning = "Successful registration. Check your e-mail.";
        require_once __SITE_PATH . "/view/login.php";
        exit();
    }

    function confirmation(){
        $confirmation_users = Player::where(array("registration_sequence" => $_GET["sequence"]));

        if(count($confirmation_users) === 0){
            $warning = "Invalid confirmation link.";
            require_once __SITE_PATH . "/view/login.php";
            exit();
        }
        else if(count($confirmation_users) > 1){
            $warning = "Something went wrong. Please contact us.";
            require_once __SITE_PATH . "/view/login.php";
            exit();
        } 

        $confirmation_user = $confirmation_users[0];

        $confirmation_user->has_registered = 1;

        $confirmation_user->update();

        $warning = "Successful registration confirmation.";
        require_once __SITE_PATH . "/view/login.php";
    }

}

?>