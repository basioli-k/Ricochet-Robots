<?php require_once __SITE_PATH . "/view" . '/_header.php';
?>
<div class="wrapper">
<div class="container">
<h1>Ricochet Robots</h1>
<form method="post" action= "<?php echo __SITE_URL; ?>/index.php?rt=login/verifyLogin" >
    <label for="user" class="label"> Username: </label>
    <input type="text" name="username" placeholder="Insert your username" class="input"> <br><br>
    <label for="password" class="label"> Password: </label>
    <input type="password" name="password" placeholder="Insert your password" class="input"> <br>
    <button type="submit" name="rt" value="homepage" class="button"> Login </button><br>
</form>
<form method="post" action="<?php echo __SITE_URL; ?>/index.php?rt=register">
    <button type="submit" value="Register" class="button">Register</button>
</form>
<br>
</div>
</div>

<?php
if(isset($warning) && strlen($warning) > 0)
    echo "<div>" . $warning . "</div>";
?>

<?php require_once __SITE_PATH . "/view" . '/_footer.php'; ?>
