<?php require_once __SITE_PATH . "/view" . '/_header.php';
?>

<form action="<?php echo __SITE_URL; ?>/index.php?rt=register/newUser" method="post">
<br>
<label for="username"> Username: </label>
<input type="text" name="username"> <br><br>
<label for="password">Password: </label> 
<input type="password" name="password"> <br><br>
<label for="email">E-mail: </label>
<input type="text" name="email"> <br><br>
<input type="submit" value="Register">

</form>

<?php
if(isset($warning) && strlen($warning) > 0)
    echo "<div>" . $warning . "</div>";
?>
<?php require_once __SITE_PATH . "/view" . '/_footer.php'; ?>
