<?php require_once __SITE_PATH . "/view" . '/_header.php';
?>

<div class="wrapper">
<div class="container">
<h1>Register</h1>
<form action="<?php echo __SITE_URL; ?>/index.php?rt=register/newUser" method="post">
<label for="username" class="label"> Username: </label>
<input type="text" name="username" placeholder="Insert your username" class="input"><br><br>
<label for="password" class="label">Password: </label> 
<input type="password" name="password" placeholder="Insert your password" class="input"> <br><br>
<label for="email" class="label">E-mail: </label>
<input type="text" name="email" class="input" placeholder="Insert your e-mail"><br><br>
<button type="submit" value="Register" class="button">Register</button>

</form>

<form action=" <?php echo __SITE_URL ?>/index.php" method="get">
    <button class="button" value="Back">Back</button>
</form>
</div>
</div>

<?php
if(isset($warning) && strlen($warning) > 0)
    echo "<div>" . $warning . "</div>";
?>
<?php require_once __SITE_PATH . "/view" . '/_footer.php'; ?>
